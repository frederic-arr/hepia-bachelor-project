#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq
#import "../data/membench.typ": *

#figure(
    label: <val-memory>,
    caption: [
        Mémoire disponible sur le système
    ],
    note: [
        Allocation mémoire maximale faisable par un conteneur sur une VM
        disposant de 256~MiB sur 100 échantillons.
    ],
    source: made-by-self,
    lq.diagram(
        width: 100%,
        xlabel: [Allocation mémoire maximale \[MiB\]],
        yaxis: (
            ticks: none,
        ),
        lq.hviolin(
            memory,
            trim: false,
        ),
    ),
)
