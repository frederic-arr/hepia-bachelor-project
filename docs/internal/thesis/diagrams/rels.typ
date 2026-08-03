#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *
#import packages.fletcher: shapes

#let static(..args) = node(shape: shapes.hexagon, ..args)
#let dyn(..args) = node(shape: shapes.parallelogram, ..args)
#let shared(..args) = node(shape: shapes.chevron, ..args)
// #let creates(parent, child, ..args) = edge(
//     parent,
//     child,
//     "-O",
//     ..args,
// )
#let owns(parent, child, ..args) = edge(
    parent,
    child,
    "-|>",
    ..args,
)
#let depends-on(from, to, ..args) = edge(
    from,
    to,
    "--<>",
    ..args,
)

#refdiagram(
    label: <rels>,
    caption: [Synthèse des types de ressources et des liens],
    note: [
        Illustre comment les différentes catégories de ressources peuvent crée
        et interagir avec d'autres ressources.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 1pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        static(label: <rels-dns>, (0, -1), title: [`network:address`])
        dyn(
            label: <rels-dnsfile>,
            (0, -2),
            title: [`system:etc-file/resolv.conf`],
        )

        static(
            label: <rels-neta>,
            (0, 0),
            title: [`network:interface/eth0`],
        )
        dyn(
            label: <rels-neta-addr>,
            (rel: (-0.5, 1), to: <rels-neta>),
            title: [`network:address`],
        )
        dyn(
            label: <rels-neta-link>,
            (rel: (0.5, 1), to: <rels-neta>),
            title: [`network:link`],
        )

        static(
            label: <rels-cona>,
            (2, 0),
            title: [container:test-a],
        )
        dyn(
            label: <rels-cona-run>,
            (rel: (-0.5, 1), to: <rels-cona>),
            title: [container-instance],
        )
        dyn(
            label: <rels-cona-img>,
            (rel: (0.75, 1), to: <rels-cona>),
            title: [image-ref],
        )

        static(
            label: <rels-conb>,
            (2, -1),
            title: [container:test-b],
        )
        dyn(
            label: <rels-conb-run>,
            (rel: (-0.5, -1), to: <rels-conb>),
            title: [container-instance],
        )
        dyn(
            label: <rels-conb-img>,
            (rel: (0.75, -1), to: <rels-conb>),
            title: [image-ref],
        )

        shared(
            label: <rels-img>,
            // num: [C.2],
            // badge-fill: blue,
            (3, -0.5),
            title: [image],
            // stroke: 2pt + blue,
        )

        node(
            label: <rels-simple>,
            num: [A],
            enclose: (<rels-dns>, <rels-dnsfile>),
            inset: 3mm,
            snap: false,
            stroke: yellow,
        )

        node(
            label: <rels-dyn>,
            num: [B],
            enclose: (<rels-neta>, <rels-neta-addr>, <rels-neta-link>),
            inset: 3mm,
            snap: false,
            stroke: yellow,
        )

        node(
            label: <rels-mut>,
            num: [C],
            enclose: (
                <rels-img>,
                <rels-cona>,
                <rels-cona-run>,
                <rels-cona-img>,
                <rels-conb>,
                <rels-conb-run>,
                <rels-conb-img>,
            ),
            inset: 3mm,
            snap: false,
            stroke: yellow,
        )

        // creates(
        //     num: [A.1],
        //     badge-fill: teal,
        //     badge-x: -2.3em,
        //     badge-y: -0.5em,
        //     stroke: teal,
        //     <rels-dns>,
        //     <rels-dnsfile>,
        //     bend: 30deg,
        // )
        owns(<rels-dns>, <rels-dnsfile>, stroke: teal, bend: -30deg)

        // creates(<rels-neta>, <rels-neta-link>, bend: -15deg)
        // creates(<rels-neta>, <rels-neta-addr>, bend: -15deg)
        owns(<rels-neta>, <rels-neta-link>, bend: 15deg)
        owns(<rels-neta>, <rels-neta-addr>, bend: 15deg)
        depends-on(
            label: <rels-neta-deps>,
            num: [B.1],
            badge-x: -1.6em,
            badge-fill: teal,
            stroke: teal,
            <rels-neta-addr>,
            <rels-neta-link>,
        )

        // creates(<rels-cona>, <rels-cona-run>, bend: -15deg)
        // creates(<rels-cona>, <rels-cona-img>, bend: -15deg)
        owns(<rels-cona>, <rels-cona-run>, bend: 15deg)
        owns(<rels-cona>, <rels-cona-img>, bend: 15deg)
        depends-on(<rels-cona-run>, <rels-cona-img>)

        // creates(<rels-conb>, <rels-conb-run>, bend: 15deg)
        // creates(<rels-conb>, <rels-conb-img>, bend: 15deg)
        owns(<rels-conb>, <rels-conb-run>, bend: -15deg)
        owns(<rels-conb>, <rels-conb-img>, bend: -15deg)
        depends-on(<rels-conb-run>, <rels-conb-img>)

        // creates(
        //     label: <cfgree-con-deps>,
        //     num: [C.1],
        //     badge-fill: teal,
        //     stroke: teal,
        //     badge-x: -1.8em,
        //     badge-y: -1.8em,
        //     <rels-cona-img>,
        //     <rels-img>,
        //     bend: 30deg,
        // )
        // creates(<rels-conb-img>, <rels-img>, bend: -30deg)
        depends-on(<rels-cona-img>, <rels-img>, bend: -30deg, stroke: teal)
        depends-on(<rels-conb-img>, <rels-img>, bend: 30deg)

        // creates(
        //     (2.75, 2.5),
        //     (3.5, 2.5),
        //     title: [Créé],
        // )
        owns(
            (2.75, 3),
            (3.5, 3),
            title: [Possède],
            floating: true,
        )
        depends-on((2.75, 3.5), (3.5, 3.5), title: [Dépend de])

        static((2, 2.5), title: [Resource statique])
        dyn((2, 3), title: [Resource dynamique])
        shared((2, 3.5), title: [Resource mutualisée])
    },
)
