#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <procstart>,
    caption: [Arbre d'exécution des processus du système],
    note: [
        Tous les processus remontnet à l'arbre parent. Le _core-controller_
        ayant connaissance étant le processus chargant la config, il sait quel
        contrôleurs doivent être lancés ou non. Chaque contrôleur lance ensuite
        les processus dont il a besoin, souvent des logiciels tiers.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(
            label: <procstart-legend-start>,
            (-1.75, -1.75),
        )

        node(
            label: <procstart-legend-fp>,
            (-1.5, -1),
            title: [first-party process],
        )
        node(
            label: <procstart-legend-tp>,
            (-1.5, -0.5),
            title: [third-party process],
            stroke: (dash: "dashed"),
        )
        edge(
            (-1.75, 0.25),
            (-1.25, 0.25),
            "-|>",
            title: "spawns child",
        )

        node(
            label: <procstart-legend-end>,
            (-1.75, 0.3),
        )

        node(
            enclose: (
                <procstart-legend-start>,
                <procstart-legend-fp>,
                <procstart-legend-tp>,
                <procstart-legend-end>,
            ),
            inset: 2mm,
            snap: false,
            title: align(top + center, place(dx: 0cm, dy: 0em)[_Legend_]),
        )

        node(label: <procstart-init>, (0, 0), title: [init])
        node(
            label: <procstart-supervisor>,
            (0, 1),
            title: [supervisor],
        )
        node(
            label: <procstart-core>,
            (0, 2),
            title: [core-controller],
        )
        node(
            label: <procstart-con>,
            (-1, 3),
            title: [container-controller],
        )
        node(
            label: <procstart-net>,
            (0, 3),
            title: [network-controller],
        )
        node(
            label: <procstart-other>,
            (1, 3),
            title: [_...-controller_],
        )
        node(
            label: <procstart-api>,
            (-1, 2),
            title: [API],
        )
        node(
            label: <procstart-rt>,
            (-1, 4),
            title: [container-runtime],
            subtitle: [Podman, Docker, ...],
            stroke: (dash: "dashed"),
        )

        node(
            label: <procstart-dhcp>,
            (-0.3, 4),
            title: [dhcp-client],
            stroke: (dash: "dashed"),
        )

        node(
            label: <procstart-ntp>,
            (0.3, 4),
            title: [ntp-client],
            stroke: (dash: "dashed"),
        )

        edge(<procstart-init>, <procstart-supervisor>, "-|>")
        edge(<procstart-supervisor>, <procstart-core>, "-|>")
        edge(<procstart-core>, <procstart-api>, "-|>")
        edge(<procstart-core>, <procstart-con>, "-|>")
        edge(<procstart-core>, <procstart-net>, "-|>")
        edge(<procstart-core>, <procstart-other>, "-|>")
        edge(<procstart-con>, <procstart-rt>, "-|>")
        edge(<procstart-net>, <procstart-dhcp>, "-|>")
        edge(<procstart-net>, <procstart-ntp>, "-|>")
    },
)
