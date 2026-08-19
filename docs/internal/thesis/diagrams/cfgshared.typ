#import "/packages.typ": *
#import "../lib.typ": *

#refdiagram(
    label: <cfgshared>,
    caption: [Problème des ressources partagées implicitement],
    note: [
        Illustre comment une ressource système va interagir avec une ressource
        concrète, et comment deux ressources concrètes différentes peuvent
        finalement interagir avec une seule et même autre ressource concrète.
    ],
    source: made-by-self,

    spacing: 1cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgshared-cfga>, (-0.75, 0), title: box(
            width: 8cm,
        )[
            ```yaml
            schema: container:instance
            name: container-a
            image: alpine:latest
            # ...
            ```
        ])

        node(label: <cfgshared-cfgb>, (0.75, 0), title: box(
            width: 8cm,
        )[
            ```yaml
            schema: container:instance
            name: container-b
            image: alpine:latest
            # ...
            ```
        ])

        node(label: <cfgshared-podimg>, (0, 3), stroke: 2pt + teal, title: [
            /var/lib/container/image/alpine/...
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
