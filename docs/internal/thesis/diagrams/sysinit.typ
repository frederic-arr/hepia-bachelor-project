#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <sysinit>,
    caption: [Démarrage du système],
    note: [TODO],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <sysinit-init>, (0, 0), title: [
            Find and mount the root partition
        ])
        edge(<sysinit-init>, <sysinit-env>, "-|>")
        node(label: <sysinit-env>, (0, 1), title: [
            Create base environment (/dev, /proc, local network interface, ...)
        ])
        edge(<sysinit-env>, <sysinit-early>, "-|>")
        node(label: <sysinit-early>, (0, 2), title: [
            Find and mount the early config
        ])
        edge(<sysinit-early>, <sysinit-cfg>, "-|>")
        node(label: <sysinit-cfg>, (0, 3), title: [
            Find, decrypt and mount full config and state
        ])
        edge(<sysinit-cfg>, <sysinit-net>, "-|>")
        node(label: <sysinit-net>, (0, 4), title: [
            Start network controller
        ])
        edge(<sysinit-net>, <sysinit-api>, "-|>")
        node(label: <sysinit-api>, (0, 5), title: [
            Start API
        ])
        edge(<sysinit-api>, <sysinit-procs>, "-|>")
        node(label: <sysinit-procs>, (0, 6), title: [
            Start other controllers
        ])
        edge(<sysinit-procs>, <sysinit-rec>, "--|>", title: [
            Wait for all controllers~~to be ready
        ])
        node(label: <sysinit-rec>, (0, 7), title: [
            Start reconciliation
        ])

        node(
            enclose: (
                <sysinit-env>,
                <sysinit-early>,
            ),
            inset: 2mm,
            title: place(dx: -2.5cm)[*supervisor*],
        )

        node(
            enclose: (
                <sysinit-cfg>,
                <sysinit-api>,
                <sysinit-procs>,
                <sysinit-rec>,
            ),
            inset: 2mm,
            title: place(dx: -3.5cm)[*core-controller*],
        )
    },
)
