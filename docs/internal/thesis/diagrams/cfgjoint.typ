#import "/packages.typ": *
#import "../lib.typ": *

#refdiagram(
    label: <cfgjoint>,
    caption: [Solution aux resources partagées],
    note: [
        Deux configuration utilisateur indépendantes, agissent au final sur une
        resource partagée au niveau du système en raison du fonctionnement de la
        resource réel.

        Note: la configuration est abrégée a des fins d'illustration
    ],
    source: made-by-self,

    spacing: 1cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgjoint-cfga>, (0.875, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-a
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-imga>, (0.875, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ImageRef
            name: container-a-img
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-runa>, (0, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-a-run
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-cfgb>, (1.85, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-b
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-imgb>, (1.85, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ImageRef
            name: container-b-img
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-runb>, (2.825, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-b-run
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-img>, (1.5, 3), title: box(
            width: 8cm,
        )[
            ```yaml
            kind: Image
            name: alpine:latest@sha256:AAAAA
            ```
        ])

        node(label: <cfgjoint-podimg>, (1.5, 4), stroke: 2pt, title: [
            /var/lib/container/image/alpine/latest
        ])

        node(label: <cfgjoint-podruna>, (0, 4), stroke: 2pt, title: [
            Running Container A
        ])

        node(label: <cfgjoint-podrunb>, (2.875, 4), stroke: 2pt, title: [
            Running Container B
        ])

        node(
            label: <cfgjoint-cfg>,
            enclose: (
                <cfgjoint-cfga>,
                <cfgjoint-cfgb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: blue,
            title: align(top + left, place(dx: 5cm, dy: 2cm, text(
                fill: blue,
            )[
                *Static Resources*
            ])),
        )

        node(
            label: <cfgjoint-dyn>,
            enclose: (
                <cfgjoint-imga>,
                <cfgjoint-runa>,
                <cfgjoint-imgb>,
                <cfgjoint-runb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(
                fill: red,
            )[
                *Dynamic Resources*
            ])),
        )

        node(
            label: <cfgjoint-real>,
            enclose: (
                <cfgjoint-podruna>,
                <cfgjoint-podrunb>,
                <cfgjoint-podimg>,
            ),
            inset: 2mm,
            snap: false,
            stroke: orange,
        )

        node(
            label: <cfgjoint-reallabel>,
            (rel: (0mm, -1cm), to: <cfgjoint-real>),
            title: text(fill: orange)[*Concrete Resources*],
        )

        node(
            label: <cfgjoint-joint>,
            num: [1],
            enclose: (
                <cfgjoint-img>,
            ),
            inset: 2mm,
            snap: false,
            stroke: fuchsia,
            title: align(top + left, place(dx: -3.5cm, dy: 0cm, text(
                fill: fuchsia,
            )[
                *Shared Resources*
            ])),
        )

        edge(<cfgjoint-cfga>, <cfgjoint-imga>, "-|>")
        edge(<cfgjoint-cfga>, <cfgjoint-runa>, "-|>")
        edge(<cfgjoint-runa>, <cfgjoint-podruna>, "-|>")
        edge(
            <cfgjoint-imga>,
            <cfgjoint-img>,
            "-|>",
            num: [2],
            stroke: yellow,
            label: <cfgjoint-imgref>,
            badge-x: 1em,
            badge-y: -0.2em,
        )

        edge(<cfgjoint-cfgb>, <cfgjoint-imgb>, "-|>")
        edge(<cfgjoint-cfgb>, <cfgjoint-runb>, "-|>")
        edge(<cfgjoint-runb>, <cfgjoint-podrunb>, "-|>")
        edge(<cfgjoint-imgb>, <cfgjoint-img>, "-|>", stroke: yellow)

        edge(
            <cfgjoint-img>,
            <cfgjoint-podimg>,
            "-|>",
            num: [3],
            stroke: yellow,
            label: <cfgjoint-noconflict>,
        )

        edge(<cfgjoint-podruna>, <cfgjoint-podimg>, "--|>")
        edge(<cfgjoint-podrunb>, <cfgjoint-podimg>, "--|>")
    },
)
