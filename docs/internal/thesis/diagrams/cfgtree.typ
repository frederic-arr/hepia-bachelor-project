#import "/packages.typ": *
#import "../lib.typ": *

#let static(..args) = node(stroke: blue, ..args)
#let dyn(..args) = node(stroke: red, ..args)
#let shared(..args) = node(stroke: fuchsia, ..args)
#let rel-rwd(parent, child, ..args) = edge(
    parent,
    child,
    "-|>",
    stroke: red,
    ..args,
)
#let rel-r(from, to, ..args) = edge(from, to, "--|>", stroke: green, ..args)
#let rel-rw(from, to, ..args) = edge(from, to, "-x-|>", stroke: fuchsia, ..args)

#refdiagram(
    label: <cfgtree>,
    caption: [Dérivation de ressources dynamiques depuis une configuration
        réseau],
    note: [
        À partir d'une unique configuration réseau, le contrôleur dérive
        automatiquement trois ressources dynamiques correspondant aux objets
        qu'il manipule au sein du noyau Linux.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 1pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        static(label: <cfgtree-dns>, (-1, 0), title: [dns])

        static(
            label: <cfgtree-neta>,
            (0, 0),
            title: [interface/eth0],
        )
        dyn(
            label: <cfgtree-neta-addr>,
            (rel: (0, 1), to: <cfgtree-neta>),
            title: [address],
        )
        dyn(
            label: <cfgtree-neta-route>,
            (rel: (-0.75, 1), to: <cfgtree-neta>),
            title: [route],
        )
        dyn(
            label: <cfgtree-neta-link>,
            (rel: (0.75, 1), to: <cfgtree-neta>),
            title: [link],
        )

        static(
            label: <cfgtree-cona>,
            (2, 0),
            title: [container/test-a],
        )
        dyn(
            label: <cfgtree-cona-run>,
            (rel: (-0.5, 1), to: <cfgtree-cona>),
            title: [container-instance],
        )
        dyn(
            label: <cfgtree-cona-img>,
            (rel: (0.75, 1), to: <cfgtree-cona>),
            title: [image-ref],
        )

        static(
            label: <cfgtree-conb>,
            (2, -1),
            title: [container/test-b],
        )
        dyn(
            label: <cfgtree-conb-run>,
            (rel: (-0.5, -1), to: <cfgtree-conb>),
            title: [container-instance],
        )
        dyn(
            label: <cfgtree-conb-img>,
            (rel: (0.75, -1), to: <cfgtree-conb>),
            title: [image-ref],
        )

        shared(
            label: <cfgtree-img>,
            (3, -0.5),
            title: [image],
        )

        rel-rwd(<cfgtree-neta>, <cfgtree-neta-link>)
        rel-rwd(<cfgtree-neta>, <cfgtree-neta-addr>)
        rel-rwd(<cfgtree-neta>, <cfgtree-neta-route>)

        rel-r(<cfgtree-neta-addr>, <cfgtree-neta-link>)
        rel-r(<cfgtree-neta-route>, <cfgtree-neta-addr>)

        rel-rwd(<cfgtree-cona>, <cfgtree-cona-run>)
        rel-rwd(<cfgtree-cona>, <cfgtree-cona-img>)
        rel-r(<cfgtree-cona-run>, <cfgtree-cona-img>)

        rel-rwd(<cfgtree-conb>, <cfgtree-conb-run>)
        rel-rwd(<cfgtree-conb>, <cfgtree-conb-img>)
        rel-r(<cfgtree-conb-run>, <cfgtree-conb-img>)

        rel-rw(<cfgtree-cona-img>, <cfgtree-img>)
        rel-rw(<cfgtree-conb-img>, <cfgtree-img>)

        rel-rwd(
            (3, 2),
            (4, 2),
            title: [Lire, écrire, supprimer],
            floating: true,
        )
        rel-r((3, 3), (4, 3), title: [Lire], floating: true)
        rel-rw((3, 4), (4, 4), title: [Lire et écrire])
    },
)
