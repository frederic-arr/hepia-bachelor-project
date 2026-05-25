#import "/lib/_utils.typ"
#show: _utils.common-config
#set par(first-line-indent: 0em)
#set text(font: "Liberation Sans", lang: "en")

#let row-label(body) = {
    set par(justify: false)

    body
}

#let row(criterion: [], relevance: "", favors: [], rationale: []) = {
    (
        row-label(criterion),
        row-label(relevance),
        row-label(favors),
        rationale,
    )
}

= Reconciliation Scheduling

*DATE*: 2026-05-25

#set heading(offset: 1)

= Summary

== Issue

To prevent configuration drift and ensure that the system is self-healing, a
reconciliation loop needs to be implemented. Each tick of the loop will
+ fetch the desired state;
+ fetch the current state;
+ act;
+ update the status.

The loop can either be implemented on the "state manager" (centralized), or
within each controller (decentralized).

== Decision

The reconciliation loop will be implemented in the state manager.

The state manager will own scheduling and orchestration, and will call
`reconcile(Resource)` for each resource. Each reconciler will return the updated
status together with any requested creations, modifications, or deletions.

= Details

== Assumptions

- The system will only managed a limited number of reconcilable resources
- If any of the controler faults, the whole system has to fault

== Constraints

- _none_

== Positions

=== Centralized Scheduling

The state manager owns the reconciliation loop. It calls `reconcile(Resource)`
on each reconciler and waits for a response containing the updated status and
any pending mutations (create / update / delete).

=== Decentralized Scheduling

Each reconciler owns its loop and implements it as it sees fit, independently
fetching desired and current state on every tick.

#pagebreak(weak: true)
#set page(flipped: true)

== Analysis

#show table.cell.where(x: 0): strong
#show table.cell.where(y: 0): strong

#table(
    columns: (12em, auto, auto, 1fr),
    row-gutter: (2.2pt, auto),
    table.header[Criterion][Relevance][Favors][Rationale],
    ..row(
        criterion: [Global scheduling control],
        relevance: [MEDIUM],
        favors: [Centralized],
        rationale: [A single loop can optimize the reconciliation by taking into
            account child and dependencies.],
    ),
    ..row(
        criterion: [Scheduling flexibility],
        relevance: [MEDIUM],
        favors: [Decentralized],
        rationale: [Each reconciler can tune its own tick interval, backoff, and
            concurrency without coordinating with a central scheduler.],
    ),
    ..row(
        criterion: [Reacting to internal state events],
        relevance: [MEDIUM],
        favors: [Centralized],
        rationale: [Event-driven reconciliation (trigger on state change rather
            than timer) is straightforward to add to a central loop;
            decentralized reconcilers would each need independent subscription
        ],
    ),
    ..row(
        criterion: [Reaction to external events],
        relevance: [MEDIUM],
        favors: [Decentralized],
        rationale: [Events originating outside the state manager (e.g. a
            container crash) can be handled directly by the owning reconciler.
            In a centralized model, all external signals must be funnelled
            through the state manager
        ],
    ),
    ..row(
        criterion: [Stuck reconciler detection],
        relevance: [LOW],
        favors: [Centralized],
        rationale: [Because the central scheduler expects a response from
            `reconcile()`, it can detect a stuck reconciler via timeout.
            Decentralized reconcilers fail silently. Minor concern given the
            small resource count.],
    ),
    ..row(
        criterion: [Automated sub-resource ownership],
        relevance: [LOW],
        favors: [Centralized],
        rationale: [Because the `reconcile()` call also returns the creation
            requests, it is trivial to associate a parent/child relationship.],
    ),
    ..row(
        criterion: [API call overhead],
        relevance: [N/A],
        favors: [Centralized],
        rationale: [`reconcile(Resource)` bundles all necessary state into one
            call vs. multiple fetches per tick in the decentralized model. Not a
            factor given no API rate-limit constraints.],
    ),
    ..row(
        criterion: [Failure blast radius],
        relevance: [N/A],
        favors: [Decentralized],
        rationale: [A bug or crash in the central scheduler halts all
            reconciliation. A failure in one decentralized reconciler is
            isolated to that resource type. Not a factor because all reconcilers
            are required to work for the system to work too.],
    ),
)

#pagebreak(weak: false)
#set page(flipped: false)

== Argument

// TODO

== Implication

- The state manager must implement the reconciliation loop and scheduling
    policy.
- Reconcilers must expose a `reconcile(Resource)` interface.
- External events must be translated forwarded to the state manager (e.g.
    `ScheduleReconcile(Identity)`).

