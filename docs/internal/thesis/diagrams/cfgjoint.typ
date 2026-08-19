#import "/packages.typ": *
#import "../lib.typ": *

#refdiagram(
    label: <cfgjoint>,
    caption: [Solution au problème du partage de ressources implicite],
    note: [
        Illustre comment, en ajoutant une abstraction, la problématique de deux
        ressources concrètes indépendantes interagissant avec une troisième
        ressource concrète peut être résolue.
    ],
    source: made-by-self,

    spacing: 1cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgjoint-cfga>, (-0.75, 0), title: box(
            width: 8cm,
        )[
            ```yaml
            schema: container:instance
            name: container-a
            image: alpine:latest
            # ...
            ```
        ])

        node(label: <cfgjoint-cfgb>, (0.75, 0), title: box(
            width: 8cm,
        )[
            ```yaml
            schema: container:instance
            name: container-b
            image: alpine:latest
            # ...
            ```
        ])

        node(label: <cfgjoint-img>, (0, 2), title: box(
            width: 8cm,
        )[
            ```yaml
            schema: container:image
            name: alpine:latest@sha256:AAA...
            ```
        ])

        node(label: <cfgjoint-podimg>, (0, 3), stroke: 2pt + teal, title: [
            /var/lib/container/image/alpine/...
        ])

        node(label: <cfgjoint-podruna>, (-1, 3), stroke: 2pt, title: [
            Running Container A
        ])

        node(label: <cfgjoint-podrunb>, (1, 3), stroke: 2pt, title: [
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
            stroke: black,
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
            stroke: black,
        )

        node(
            label: <cfgjoint-reallabel>,
            (rel: (0mm, -1cm), to: <cfgjoint-real>),
            title: [*Concrete Resources*],
        )

        node(
            label: <cfgjoint-joint>,
            num: [1],
            enclose: (
                <cfgjoint-img>,
            ),
            inset: 2mm,
            snap: false,
            stroke: yellow,
        )

        edge(<cfgjoint-cfga>, <cfgjoint-podruna>, "-|>")
        edge(
            <cfgjoint-cfga>,
            <cfgjoint-img>,
            "-|>",
            num: [2],
            stroke: yellow,
            label: <cfgjoint-imgref>,
            badge-x: -0.6em,
            badge-y: -0.8em,
        )

        edge(<cfgjoint-cfgb>, <cfgjoint-podrunb>, "-|>")
        edge(<cfgjoint-cfgb>, <cfgjoint-img>, "-|>", stroke: yellow)

        edge(
            <cfgjoint-img>,
            <cfgjoint-podimg>,
            "-|>",
            num: [3],
            stroke: teal,
            badge-fill: teal,
            label: <cfgjoint-noconflict>,
        )

        edge(<cfgjoint-podruna>, <cfgjoint-podimg>, "--|>")
        edge(<cfgjoint-podrunb>, <cfgjoint-podimg>, "--|>")
    },
)
