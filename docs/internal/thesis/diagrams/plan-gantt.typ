#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.timeliney as timeliney

#let cell(
    from: (0, 1),
    to: (1, 1),
    color: rgb("159fec"),
    init: none,
    name,
    ..more,
) = {
    let ln = (
        from: (from.at(0) - 1) * 5 + from.at(1) - 1,
        to: (to.at(0) - 1) * 5 + to.at(1),
        style: (stroke: 8pt + color),
    )

    let more = more
        .pos()
        .map(x => (
            from: (x.from.at(0) - 1) * 5 + x.from.at(1) - 1,
            to: (x.to.at(0) - 1) * 5 + x.to.at(1),
            style: (stroke: 8pt + color),
        ))

    if init == none {
        timeliney.task(
            align(left, name),
            ln,
            ..more,
        )
    } else {
        let from = init.from
        let to = init.to
        timeliney.task(
            name,
            (
                from: (from.at(0) - 1) * 5 + from.at(1) - 1,
                to: (to.at(0) - 1) * 5 + to.at(1),
                style: (stroke: 8pt + rgb("159fec")),
            ),
            ln,
            ..more,
        )
    }
}

#let in-time(from: (0, 1), to: (1, 1), init: none, name, ..more) = {
    cell(from: from, to: to, color: rgb("34a853"), init: init, name, ..more)
}

#let not-done(from: (0, 1), to: (1, 1), init: none, name, ..more) = {
    cell(from: from, to: to, color: rgb("ea4335"), init: init, name, ..more)
}

#let longer(from: (0, 1), to: (1, 1), init: none, name, ..more) = {
    cell(from: from, to: to, color: rgb("fbbc04"), init: init, name, ..more)
}

#figure(
    label: <plan-gantt>,
    caption: [Planification du travail],
    note: [
        Planification du travail sous la forme d'un diagramme de Gantt. En vert
        les tâches dont la planification de la durée est correcte. Si elles
        n'ont pas été faites au moment prévu, le moment prévu est superposé en
        bleu. En rouge les tâches non effectuées. En orange, les tâches ayant
        duré plus longtemps que prévu.
    ],
    source: made-by-self,
    {
        set text(size: 8pt)

        timeliney.timeline(
            show-grid: true,
            {
                import timeliney: *

                headerline(
                    ..range(11).map(n => group((strong("S" + str(n + 1)), 5))),
                )

                in-time(
                    "Énoncé",
                    from: (1, 1),
                    to: (1, 5),
                )

                in-time(
                    "Conception Architecture de base",
                    from: (2, 1),
                    to: (3, 5),
                )
                in-time("Bases gestion réseau", from: (3, 2), to: (3, 2))
                in-time(
                    "Bases gestion conteneurs",
                    from: (3, 3),
                    to: (3, 5),
                )
                in-time(
                    "Standalone OS packaging (no Debian, etc.)",
                    from: (4, 1),
                    to: (4, 4),
                )
                in-time(
                    "Gestion complète des conteneurs",
                    from: (4, 4),
                    to: (5, 4),
                )
                in-time(
                    "Gestion complète réseau",
                    from: (5, 5),
                    to: (6, 2),
                )

                in-time("API Externe", from: (7, 2), to: (7, 3), init: (
                    from: (6, 3),
                    to: (6, 4),
                ))

                in-time("Client API en CLI", from: (7, 2), to: (7, 3), init: (
                    from: (6, 3),
                    to: (6, 4),
                ))

                in-time(
                    "Installation de l'OS",
                    from: (7, 4),
                    to: (7, 5),
                    init: (
                        from: (6, 5),
                        to: (7, 3),
                    ),
                )
                in-time(
                    "Provider Terraform",
                    from: (11, 5),
                    to: (11, 5),
                    init: (
                        from: (7, 5),
                        to: (7, 5),
                    ),
                )
                in-time(
                    "Gestion stockage (BTRFS, LUKS)",
                    from: (11, 4),
                    to: (11, 5),
                    init: (
                        from: (8, 1),
                        to: (9, 1),
                    ),
                )

                not-done("Job Scheduling/CRON", from: (9, 2), to: (9, 2))
                not-done(
                    "Bundling de la config et des conteneurs",
                    from: (9, 3),
                    to: (9, 4),
                )
                not-done("Logging et monitorinog", from: (9, 5), to: (9, 5))
                longer(
                    "Test et validation",
                    from: (10, 4),
                    to: (11, 3),
                    init: (
                        from: (10, 1),
                        to: (10, 3),
                    ),
                    (
                        from: (8, 2),
                        to: (9, 2),
                    ),
                )
                longer(
                    "Amélioration du code",
                    from: (9, 2),
                    to: (10, 3),
                    init: (
                        from: (11, 2),
                        to: (11, 5),
                    ),
                )

                milestone(
                    at: 6 * 5 + 0.5,
                    style: (stroke: (dash: "dashed")),
                    align(center, [
                        *Rendu intermédiaire*\
                        29.06
                    ]),
                )
            },
        )
    },
)
