#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <cfgdyn>,
    caption: [Création de sous-ressources à partir d'un parent],
    note: [
        Illustre comment il est possible d'abstraire la gestion complexe de
        plusieurs ressources derrière une interface simple.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgdyn-cfg>, num: [1], (1, 0), title: box(width: 6cm)[
            #codly(
                header: [Original resource],
                highlighted-lines: (
                    (2, aqua.lighten(60%)),
                    (3, green.lighten(60%)),
                    (4, yellow.lighten(60%)),
                ),
            )
            ```yaml
            kind: Network
            name: eth0
            up: true
            address: 10.194.1.42/24
            ```
        ])

        node(label: <cfgdyn-link>, (0, 1), title: box(
            width: 6cm,
        )[
            #codly(
                highlighted-lines: (
                    (3, aqua.lighten(60%)),
                    (4, green.lighten(60%)),
                ),
            )
            ```yaml
            kind: Link
            name: dyn-eth0-link
            match: eth0
            up: true
            ```
        ])

        node(label: <cfgdyn-addr>, (0.875, 2), title: box(
            width: 6cm,
        )[
            #codly(
                highlighted-lines: (
                    (3, aqua.lighten(60%)),
                    (4, yellow.lighten(60%)),
                ),
            )
            ```yaml
            kind: Address
            name: dyn-eth0-addr
            link: eth0
            address: 10.194.1.42/24
            ```
        ])

        node(label: <cfgdyn-rte>, (0, 2), title: box(
            width: 6cm,
        )[
            #codly(
                highlighted-lines: (
                    (3, yellow.lighten(60%)),
                    (4, yellow.lighten(60%)),
                ),
            )
            ```yaml
            kind: Route
            name: dyn-eth0-rte
            network: 0.0.0.0/0
            via: 10.194.1.1
            ```
        ])

        node(
            label: <cfgdyn-dyn>,
            num: [2],
            enclose: (<cfgdyn-link>, <cfgdyn-addr>, <cfgdyn-rte>),
            inset: 2mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Sub-Resources*
            ])),
        )

        edge(<cfgdyn-cfg>, <cfgdyn-link>, "-|>")
        edge(<cfgdyn-cfg>, <cfgdyn-addr>, "-|>")
        edge(<cfgdyn-cfg>, <cfgdyn-rte>, "-|>")
    },
)
