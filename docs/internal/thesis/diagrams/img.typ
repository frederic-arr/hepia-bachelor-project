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
        node(label: <img-iso>, (0, 0), title: [ISO])
        node(label: <img-initrd>, (-1, 1), title: [initrd])
        node(label: <img-init>, (-2, 1), title: [`/init`])
        node(label: <img-rootfs>, (0, 1), title: [rootfs])
        node(label: <img-kernel>, (-1, 0), title: [kernel])
        node(label: <img-supervisor>, (-1, 2), title: [supervisor])
        node(label: <img-net>, (0, 2), title: [network-controller])
        node(label: <img-con>, (1, 2), title: [container-controller])
        node(label: <img-sys>, (1, 1), title: [system-controller])
        node(label: <img-pod>, (1, 0), title: [Podman])

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
