#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <sysinit>,
    caption: [Démarrage du système],
    note: [
        Illustre les étapes principales du démarrage du système, avant que
        celui-ci n'entame la réconciliation.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <sysinit-init>, (0, 0), title: [
            Find and mount the real root fs (SquashFS, Tmpfs, OverlayFS)
        ])

        edge(<sysinit-init>, <sysinit-env>, "-|>")
        node(label: <sysinit-env>, (0, 1), title: [
            Create base environment (/dev, /proc, local network interface, ...)
        ])
        edge(<sysinit-env>, <sysinit-early>, "-|>")
        node(label: <sysinit-early>, (0, 2), title: [
            Find and mount additional volumes (`/config`, `/var`, ...)
        ])
        edge(<sysinit-early>, <sysinit-con>, "-|>")
        node(label: <sysinit-con>, (0, 4), title: [
            Start the controllers
        ])
        edge(<sysinit-con>, <sysinit-orch>, "--|>", title: [
            Wait for all controller to be ready
        ])
        node(label: <sysinit-orch>, (0, 5), title: [
            Start the orchestrator
        ])

        node(
            enclose: (
                <sysinit-env>,
                <sysinit-early>,
                <sysinit-orch>,
            ),
            inset: 2mm,
            stroke: red,
            title: place(dx: -2.5cm, text(fill: red)[*supervisor*]),
        )
    },
)
