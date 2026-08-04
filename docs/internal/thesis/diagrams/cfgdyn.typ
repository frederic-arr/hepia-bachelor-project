#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <cfgdyn>,
    caption: [Création de sous-ressources à partir d'une configuration
        utilisateur
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
                    (3, yellow.lighten(60%)),
                ),
            )
            ```yaml
            schema: network:interface
            name: eth0
            address: 10.0.2.15/24
            ```
        ])

        node(label: <cfgdyn-link>, (0.875, 2), title: box(
            width: 7cm,
        )[
            #codly(
                highlighted-lines: (
                    (2, aqua.lighten(60%)),
                ),
            )
            ```yaml
            schema: network:link
            name: eth0
            admin_up: true
            ```
        ])

        node(label: <cfgdyn-addr>, (0, 1), title: box(
            width: 8cm,
        )[
            #codly(
                highlighted-lines: (
                    (3, aqua.lighten(60%)),
                    (4, yellow.lighten(60%)),
                    (5, yellow.lighten(60%)),
                ),
            )
            ```yaml
            schema: network:address
            name: dyn-eth0-iface
            link: eth0
            address: 10.0.2.15
            prefix_len: 24
            ```
        ])

        node(label: <cfgdyn-rte>, (0, 2), title: box(
            width: 7cm,
        )[
            ```yaml
            schema: network:route
            name: dyn-eth0-iface
            destination: 0.0.0.0
            prefix_len: 0
            via: 10.0.2.1
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
