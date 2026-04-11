---
tags: [philosophy]
---
# What Converge Is Not

This page exists to prevent drift. When someone proposes a feature or pattern, check it against this list. If it turns Converge into one of these things, the proposal is wrong.

## Not a Chatbot Framework

Converge does not manage conversations, threads, or message history. It does not have a concept of "user message" and "assistant response." If you need a chatbot, use a chatbot framework. If you need a chatbot that makes governed decisions, the chatbot calls Converge — Converge does not become the chatbot.

## Not a Workflow Engine

There are no steps, stages, or directed edges between agents. Agents do not "pass work" to the next agent. The engine runs all eligible agents each cycle and detects convergence. If you find yourself drawing arrows between agents, you are thinking about this wrong.

## Not a Prompt Orchestration System

Converge is not a wrapper around LLM calls. Not every agent needs an LLM. Rule-based agents, optimization agents, and analytics agents are equally first-class. The engine does not know or care whether an agent uses an LLM internally.

## Not an Actor Model

Agents do not send messages to each other. There are no mailboxes, no channels, no pub/sub. All communication happens through the shared context. If two agents need to coordinate, they both read and write to the same context keys. The engine handles ordering.

## Not an Event-Driven System

There is no event bus, no subscribers, no event sourcing of agent-to-agent communication. The context is not a log of events — it is the current state of accumulated evidence. Agents read the current context, not a stream of changes.

## Not a Database

The context is an in-memory, append-only evidence store scoped to a single convergence run. It is not a persistent database. Facts exist for the duration of the run. If you need persistence, project converged facts into your own storage after the run completes.

## Anti-Patterns to Reject

| Proposal | Why it's wrong | Axiom violated |
|---|---|---|
| "Agent A sends a message to Agent B" | Agents communicate through context, not messages | Axiom 2: Convergence Over Control Flow |
| "Let agents mutate facts" | Facts are append-only; corrections are new facts | Axiom 3: Append-Only Truth |
| "Skip the promotion gate for trusted agents" | No implicit permissions; all proposals go through the gate | Axiom 1: Explicit Authority |
| "Add retry logic to the engine" | No hidden work; failures are visible | Axiom 8: No Hidden Work |
| "Let agents call each other" | All coordination through shared context | Axiom 2: Convergence Over Control Flow |
| "Auto-approve low-confidence proposals" | Confidence thresholds are governance policy, not convenience | Axiom 4: Agents Suggest, Engine Decides |
| "Add a message queue between cycles" | No event-driven patterns; context is state, not a stream | Axiom 2: Convergence Over Control Flow |
| "Relax invariants for performance" | Safety by construction is non-negotiable | Axiom 5: Safety by Construction |

See also: [[Philosophy/Nine Axioms]], [[Philosophy/Why Converge]]
