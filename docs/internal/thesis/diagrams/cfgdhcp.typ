#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <cfgdhcp>,
    caption: [Création de sous-ressources à partir d'une configuration DHCP],
    source: made-by-self,

    spacing: 1.2cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgdhcp-cfg>, num: [1], (1, 0), title: box(width: 6cm)[
            #codly(
                header: [Static resource],
            )
            ```yaml
            schema: network:dhcp
            name: eth0
            ```
        ])

        node(label: <cfgdhcp-addr>, (0, 1), title: box(
            width: 8cm,
        )[
            ```yaml
            schema: network:address
            name: dyn-eth0-dhcp
            link: eth0
            address: 10.0.2.15
            prefix_len: 24
            ```
        ])

        node(label: <cfgdhcp-rte>, (0.875, 2), title: box(
            width: 7cm,
        )[
            ```yaml
            schema: network:route
            name: dyn-eth0-dhcp
            destination: 0.0.0.0
            prefix_len: 0
            gateway: 10.0.2.1
            ```
        ])

        node(
            label: <cfgdhcp-dyn>,
            num: [2],
            enclose: (<cfgdhcp-addr>, <cfgdhcp-rte>),
            inset: 2mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Sub-Resources*
            ])),
        )

        edge(<cfgdhcp-cfg>, <cfgdhcp-addr>, "-|>")
        edge(<cfgdhcp-cfg>, <cfgdhcp-rte>, "-|>")
    },
)
