#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <img>,
    caption: [Arbre de build],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(
            label: <build-legend-start>,
            (-1.75, -1.75),
        )

        node(
            label: <build-legend-fp>,
            (-1.5, -1),
            title: [images/archives],
            stroke: teal,
        )
        node(
            label: <build-legend-tp>,
            (-1.5, -0.5),
            title: [first-party Rust programs],
            stroke: red,
        )
        node(
            label: <build-legend-tp>,
            (-1.5, -0),
            title: [third-party programs],
            stroke: yellow,
        )

        node(
            label: <build-legend-end>,
            (-1.75, 0.3),
        )

        node(
            enclose: (
                <build-legend-start>,
                <build-legend-fp>,
                <build-legend-tp>,
                <build-legend-end>,
            ),
            inset: 2mm,
            snap: false,
            title: align(top + center, place(dx: 0cm, dy: 0em)[_Legend_]),
        )

        node(label: <img-iso>, (0, 1), title: [ISO], stroke: teal)
        node(label: <img-initrd>, (-1, 2), title: [initrd], stroke: teal)
        node(label: <img-init>, (-2, 2), title: [`/init`], stroke: red)
        node(label: <img-rootfs>, (0, 2), title: [rootfs], stroke: teal)
        node(label: <img-kernel>, (-1, 1), title: [kernel], stroke: teal)
        node(label: <img-supervisor>, (-1, 3), title: [supervisor], stroke: red)
        node(label: <img-net>, (0, 3), title: [network-controller], stroke: red)
        node(
            label: <img-con>,
            (1, 3),
            title: [container-controller],
            stroke: red,
        )
        node(label: <img-sys>, (1, 2), title: [system-controller], stroke: red)
        node(label: <img-pod>, (1, 1), title: [Podman], stroke: yellow)

        edge(<img-iso>, <img-rootfs>, "-|>")
        edge(<img-iso>, <img-kernel>, "-|>")
        edge(<img-iso>, <img-initrd>, "-|>")
        edge(<img-initrd>, <img-init>, "-|>")
        edge(<img-rootfs>, <img-supervisor>, "-|>")
        edge(<img-rootfs>, <img-net>, "-|>")
        edge(<img-rootfs>, <img-con>, "-|>")
        edge(<img-rootfs>, <img-sys>, "-|>")
        edge(<img-rootfs>, <img-pod>, "-|>")
    },
)
