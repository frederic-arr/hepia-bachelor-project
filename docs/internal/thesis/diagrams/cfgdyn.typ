#import "/packages.typ": *
#import "../lib.typ": *

#import packages.codly: *

#refdiagram(
    label: <cfgdyn>,
    caption: [Dérivation de ressources dynamiques depuis une configuration
        réseau],
    note: [
        À partir d'une unique configuration réseau, le contrôleur dérive
        automatiquement trois ressources dynamiques correspondant aux objets
        qu'il manipule au sein du noyau Linux.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgdyn-cfg>, num: [1], (0, 0), title: box(width: 6cm)[
            #codly(
                header: [User Configuration],
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

        node(label: <cfgdyn-link>, (1, 1), title: box(
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
            num: [2],
            enclose: (<cfgdyn-link>, <cfgdyn-addr>, <cfgdyn-rte>),
            inset: 2mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Dynamic Resources*
            ])),
        )

        edge(<cfgdyn-cfg>, <cfgdyn-link>, "-|>")
        edge(<cfgdyn-cfg>, <cfgdyn-addr>, "-|>")
        edge(<cfgdyn-cfg>, <cfgdyn-rte>, "-|>")
    },
)
