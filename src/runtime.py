from __future__ import annotations

from dataclasses import dataclass

from .commands import PORTED_COMMANDS
from .context import PortContext, build_port_context, render_context
from .execution_registry import build_execution_registry
from .history import HistoryLog
from .models import PermissionDenial, PortingModule
from .query_engine import QueryEngineConfig, QueryEnginePort, TurnResult
from .setup import SetupReport, WorkspaceSetup, run_setup
from .system_init import build_system_init_message
from .tools import PORTED_TOOLS


@dataclass(frozen=True)
class RoutedMatch:
    kind: str
    name: str
    source_hint: str
    score: int


@dataclass
class RuntimeSession:
    prompt: str
    context: PortContext
    setup: WorkspaceSetup
    setup_report: SetupReport
    system_init_message: str
    history: HistoryLog
    routed_matches: list[RoutedMatch]
    turn_result: TurnResult
    command_execution_messages: tuple[str, ...]
    tool_execution_messages: tuple[str, ...]
    stream_events: tuple[dict[str, object], ...]
    persisted_session_path: str

    def as_markdown(self) -> str:
        lines = [
            "# Runtime Session",
            "",
            f"Prompt: {self.prompt}",
            "",
            "## Context",
            render_context(self.context),
            "",
            "## Setup",
            f"- Python: {self.setup.python_version} ({self.setup.implementation})",
            f"- Platform: {self.setup.platform_name}",
            f"- Test command: {self.setup.test_command}",
            "",
            "## Startup Steps",
            *(f"- {step}" for step in self.setup.startup_steps()),
            "",
            "## System Init",
            self.system_init_message,
            "",
            "## Routed Matches",
        ]
        if self.routed_matches:
            lines.extend(
                f"- [{match.kind}] {match.name} ({match.score}) — {match.source_hint}" for match in self.routed_matches
            )
        else:
            lines.append("- none")
        lines.extend(
            [
                "",
                "## Command Execution",
                *(self.command_execution_messages or ("none",)),
                "",
                "## Tool Execution",
                *(self.tool_execution_messages or ("none",)),
                "",
                "## Stream Events",
                *(f"- {event['type']}: {event}" for event in self.stream_events),
                "",
                "## Turn Result",
                self.turn_result.output,
                "",
                f"Persisted session path: {self.persisted_session_path}",
                "",
                self.history.as_markdown(),
            ]
        )
        return "\n".join(lines)


class PortRuntime:
    def route_prompt(self, prompt: str, limit: int = 5) -> list[RoutedMatch]:
        tokens = {token.lower() for token in prompt.replace("/", " ").replace("-", " ").split() if token}
        by_kind = {
            "command": self._collect_matches(tokens, PORTED_COMMANDS, "command"),
            "tool": self._collect_matches(tokens, PORTED_TOOLS, "tool"),
        }

        selected: list[RoutedMatch] = []
        for kind in ("command", "tool"):
            if by_kind[kind]:
                selected.append(by_kind[kind].pop(0))

        leftovers = sorted(
            [match for matches in by_kind.values() for match in matches],
            key=lambda item: (-item.score, item.kind, item.name),
        )
        selected.extend(leftovers[: max(0, limit - len(selected))])
        return selected[:limit]

    def bootstrap_session(self, prompt: str, limit: int = 5) -> RuntimeSession:
        context = build_port_context()
        setup_report = run_setup(trusted=True)
        setup = setup_report.setup
        history = HistoryLog()
        engine = QueryEnginePort.from_workspace()
        history.add(
            "context", f"python_files={context.python_file_count}, archive_available={context.archive_available}"
        )
        history.add("registry", f"commands={len(PORTED_COMMANDS)}, tools={len(PORTED_TOOLS)}")
        matches = self.route_prompt(prompt, limit=limit)
        registry = build_execution_registry()
        command_execs = tuple(
            cmd.execute(prompt)
            for match in matches
            if match.kind == "command" and (cmd := registry.command(match.name)) is not None
        )
        tool_execs = tuple(
            tool.execute(prompt)
            for match in matches
            if match.kind == "tool" and (tool := registry.tool(match.name)) is not None
        )
        denials = tuple(self._infer_permission_denials(matches))
        matched_command_names = tuple(match.name for match in matches if match.kind == "command")
        matched_tool_names = tuple(match.name for match in matches if match.kind == "tool")
        turn_result = engine.submit_message(
            prompt,
            matched_commands=matched_command_names,
            matched_tools=matched_tool_names,
            denied_tools=denials,
        )
        stream_events = self._build_stream_events(
            engine=engine,
            prompt=prompt,
            matched_commands=matched_command_names,
            matched_tools=matched_tool_names,
            denied_tools=denials,
            turn_result=turn_result,
        )
        persisted_session_path = engine.persist_session()
        history.add("routing", f"matches={len(matches)} for prompt={prompt!r}")
        history.add("execution", f"command_execs={len(command_execs)} tool_execs={len(tool_execs)}")
        history.add(
            "turn",
            f"commands={len(turn_result.matched_commands)} tools={len(turn_result.matched_tools)} denials={len(turn_result.permission_denials)} stop={turn_result.stop_reason}",
        )
        history.add("session_store", persisted_session_path)
        return RuntimeSession(
            prompt=prompt,
            context=context,
            setup=setup,
            setup_report=setup_report,
            system_init_message=build_system_init_message(trusted=True),
            history=history,
            routed_matches=matches,
            turn_result=turn_result,
            command_execution_messages=command_execs,
            tool_execution_messages=tool_execs,
            stream_events=stream_events,
            persisted_session_path=persisted_session_path,
        )

    def run_turn_loop(
        self, prompt: str, limit: int = 5, max_turns: int = 3, structured_output: bool = False
    ) -> list[TurnResult]:
        engine = QueryEnginePort.from_workspace()
        engine.config = QueryEngineConfig(max_turns=max_turns, structured_output=structured_output)
        matches = self.route_prompt(prompt, limit=limit)
        command_names = tuple(match.name for match in matches if match.kind == "command")
        tool_names = tuple(match.name for match in matches if match.kind == "tool")
        results: list[TurnResult] = []
        for turn in range(max_turns):
            turn_prompt = prompt if turn == 0 else f"{prompt} [turn {turn + 1}]"
            result = engine.submit_message(turn_prompt, command_names, tool_names, ())
            results.append(result)
            if result.stop_reason != "completed":
                break
        return results

    def _build_stream_events(
        self,
        *,
        engine: QueryEnginePort,
        prompt: str,
        matched_commands: tuple[str, ...],
        matched_tools: tuple[str, ...],
        denied_tools: tuple[PermissionDenial, ...],
        turn_result: TurnResult,
    ) -> tuple[dict[str, object], ...]:
        events: list[dict[str, object]] = [
            {"type": "message_start", "session_id": engine.session_id, "prompt": prompt},
        ]
        if matched_commands:
            events.append({"type": "command_match", "commands": matched_commands})
        if matched_tools:
            events.append({"type": "tool_match", "tools": matched_tools})
        if denied_tools:
            events.append({"type": "permission_denial", "denials": [denial.tool_name for denial in denied_tools]})
        events.append({"type": "message_delta", "text": turn_result.output})
        events.append(
            {
                "type": "message_stop",
                "usage": {
                    "input_tokens": turn_result.usage.input_tokens,
                    "output_tokens": turn_result.usage.output_tokens,
                },
                "stop_reason": turn_result.stop_reason,
                "transcript_size": len(engine.transcript_store.entries),
            }
        )
        return tuple(events)

    def _infer_permission_denials(self, matches: list[RoutedMatch]) -> list[PermissionDenial]:
        denials: list[PermissionDenial] = []
        for match in matches:
            if match.kind == "tool" and "bash" in match.name.lower():
                denials.append(
                    PermissionDenial(
                        tool_name=match.name,
                        reason="destructive shell execution remains gated in the Python port",
                    )
                )
        return denials

    def _collect_matches(self, tokens: set[str], modules: tuple[PortingModule, ...], kind: str) -> list[RoutedMatch]:
        matches: list[RoutedMatch] = []
        for module in modules:
            score = self._score(tokens, module)
            if score > 0:
                matches.append(RoutedMatch(kind=kind, name=module.name, source_hint=module.source_hint, score=score))
        matches.sort(key=lambda item: (-item.score, item.name))
        return matches

    @staticmethod
    def _score(tokens: set[str], module: PortingModule) -> int:
        haystacks = [module.name.lower(), module.source_hint.lower(), module.responsibility.lower()]
        score = 0
        for token in tokens:
            if any(token in haystack for haystack in haystacks):
                score += 1
        return score
