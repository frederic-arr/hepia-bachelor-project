#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <procstart>,
    caption: [Arbre d'exécution des processus du système],
    note: [
        Illustre l'ordre dans lequel les processus du système sont démarrés, et
        qui les lance.
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
            (-1, 1),
            title: [state-manager],
        )
        node(
            label: <procstart-con>,
            (-1, 2),
            title: [container-controller],
        )
        node(
            label: <procstart-net>,
            (0, 2),
            title: [network-controller],
        )
        node(
            label: <procstart-other>,
            (1, 2),
            title: [system-controller],
        )
        node(
            label: <procstart-rt>,
            (-1, 3),
            title: [container-runtime],
            subtitle: [Podman, Docker, ...],
            stroke: (dash: "dashed"),
        )

        node(
            label: <procstart-dhcp>,
            (0, 3),
            title: [dhcp-client],
            stroke: (dash: "dashed"),
        )

        edge(<procstart-init>, <procstart-supervisor>, "-|>")
        edge(<procstart-supervisor>, <procstart-core>, "-|>")
        edge(<procstart-supervisor>, <procstart-con>, "-|>")
        edge(<procstart-supervisor>, <procstart-net>, "-|>")
        edge(<procstart-supervisor>, <procstart-other>, "-|>")
        edge(<procstart-con>, <procstart-rt>, "-|>")
        edge(<procstart-net>, <procstart-dhcp>, "-|>")
    },
)
