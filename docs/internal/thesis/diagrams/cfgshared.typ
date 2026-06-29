#import "/packages.typ": *
#import "../lib.typ": *

#refdiagram(
    label: <cfgshared>,
    caption: [Problème des ressources partagées implicitement],
    note: [
        Illustre comment une ressource système va interagir avec une ressource
        concrète, et comment deux ressources concrètes différentes peuvent
        finalement interagir avec une seule et même autre ressource concrète.

        Note: la configuration est abrégée a des fins d'illustration
    ],
    source: made-by-self,

    spacing: 1cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgshared-cfga>, (-1, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-a
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-cfgb>, (1, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-b
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-podimg>, (0, 3), stroke: 2pt + teal, title: [
            /var/lib/container/image/alpine/latest
        ])

        node(label: <cfgshared-podruna>, (-1, 3), stroke: 2pt, title: [
            Running Container A
        ])

        node(label: <cfgshared-podrunb>, (1, 3), stroke: 2pt, title: [
            Running Container B
        ])

        node(
            label: <cfgshared-cfg>,
            enclose: (
                <cfgshared-cfga>,
                <cfgshared-cfgb>,
            ),
            inset: 2mm,
            snap: false,
            num: [1],
            stroke: yellow,
        )

        node(
            label: <cfgshared-real>,
            enclose: (
                <cfgshared-podruna>,
                <cfgshared-podrunb>,
                <cfgshared-podimg>,
            ),
            inset: 2mm,
            snap: false,
            num: [2],
            stroke: yellow,
        )

        node(
            label: <cfgshared-reallabel>,
            (rel: (0mm, -1cm), to: <cfgshared-real>),
            title: [*Concrete Resources*],
        )

        edge(<cfgshared-cfga>, <cfgshared-podruna>, "-|>")
        edge(<cfgshared-cfgb>, <cfgshared-podrunb>, "-|>")

        edge(
            <cfgshared-podruna>,
            <cfgshared-podimg>,
            "--|>",
            stroke: teal,
            label: <cfgshared-conflict>,
            num: [3],
            badge-fill: teal,
        )
        edge(<cfgshared-podrunb>, <cfgshared-podimg>, "--|>", stroke: teal)
    },
)


/*
#refdiagram(
    label: <cfgshared>,
    caption: [Problème de resources partagées],
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
        node(label: <cfgshared-cfga>, (0.875, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-a
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-imga>, (0.875, 2), title: box(
            width: 6cm,
            hide[
                ```yaml
                kind: Image
                name: container-a-img
                image: alpine:latest
                ```
            ],
        ))

        node(label: <cfgshared-runa>, (0, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-a-run
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-cfgb>, (1.85, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-b
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-imgb>, (1.85, 2), title: box(
            width: 6cm,
            hide[
                ```yaml
                kind: Image
                name: container-b-img
                image: alpine:latest
                ```
            ],
        ))

        node(label: <cfgshared-runb>, (2.825, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-b-run
            image: alpine:latest
            ```
        ])

        // Placeholder to occupy the space
        node((1.5, 3), title: box(
            width: 8cm,
            hide[
                ```yaml
                kind: Image
                name: alpine:latest@sha256:AAAAA
                ```
            ],
        ))

        node(label: <cfgshared-podimg>, (1.5, 4), stroke: 2pt + teal, title: [
            /var/lib/container/image/alpine/latest
        ])

        node(label: <cfgshared-podruna>, (0, 4), stroke: 2pt, title: [
            Running Container A
        ])

        node(label: <cfgshared-podrunb>, (2.875, 4), stroke: 2pt, title: [
            Running Container B
        ])

        node(
            label: <cfgshared-cfg>,
            num: [1],
            enclose: (
                <cfgshared-cfga>,
                <cfgshared-cfgb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: yellow,
            // stroke: blue,
            // title: align(top + left, place(dx: 5cm, dy: 2cm, text(
            //     fill: blue,
            // )[
            //     *Static Resources*
            // ])),
        )

        node(
            label: <cfgshared-dyn>,
            num: [2],
            enclose: (
                <cfgshared-imga>,
                <cfgshared-runa>,
                <cfgshared-imgb>,
                <cfgshared-runb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: yellow,
            // stroke: red,
            // title: align(top + left, place(dx: -5mm, dy: -10mm, text(
            //     fill: red,
            // )[
            //     *Dynamic Resources*
            // ])),
        )

        node(
            label: <cfgshared-real>,
            num: [3],
            enclose: (
                <cfgshared-podruna>,
                <cfgshared-podrunb>,
                <cfgshared-podimg>,
            ),
            inset: 2mm,
            snap: false,
            stroke: yellow,
        )

        node(
            label: <cfgshared-reallabel>,
            (rel: (0mm, -1cm), to: <cfgshared-real>),
            title: [*Concrete Resources*],
        )

        // edge(<cfgshared-cfga>, <cfgshared-imga>, "-|>")
        edge(<cfgshared-cfga>, <cfgshared-runa>, "-|>")
        edge(<cfgshared-runa>, <cfgshared-podruna>, "-|>")
        // edge(<cfgshared-imga>, <cfgshared-podimg>, "-|>")

        // edge(<cfgshared-cfgb>, <cfgshared-imgb>, "-|>")
        edge(<cfgshared-cfgb>, <cfgshared-runb>, "-|>")
        edge(<cfgshared-runb>, <cfgshared-podrunb>, "-|>")
        // edge(<cfgshared-imgb>, <cfgshared-podimg>, "-|>")

        edge(
            <cfgshared-podruna>,
            <cfgshared-podimg>,
            "--|>",
            label: <cfgshared-conflict>,
            num: [4],
            stroke: teal,
            badge-fill: teal,
        )
        edge(<cfgshared-podrunb>, <cfgshared-podimg>, "--|>", stroke: teal)
    },
)
*/
