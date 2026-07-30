#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq
#import "../data/time-noinstall.typ": *

#let lim = calc.max(..time_to_dhcp)
#let lim = calc.ceil(lim / 1000) * 1000

#figure(
    label: <val-boot-time-noinstall>,
    caption: [
        Temps jusqu'au ce que l'API soit accessible
    ],
    note: [
        Temps entre le démarrage du noyeau (après le bootloader) et la
        configuration d'une route via DHCP. L'API devient accessible à partir de
        ce moment. \
        Taille de l'échantillion: 100
    ],
    source: made-by-self,
    lq.diagram(
        width: 100%,
        xlabel: [Temps \[ms\]],
        xaxis: (
            exponent: 3,
            lim: (0, lim),
        ),
        yaxis: (
            ticks: none,
        ),
        lq.hviolin(
            delta(time_to_init, time_to_kernel),
            trim: false,
            label: [Time until /init],
        ),
        lq.hviolin(
            delta(time_to_dhcp, time_to_kernel),
            trim: false,
            label: [Time until DHCP route recieved],
            side: "low",
        ),
        lq.hviolin(
            delta(time_to_dhcp, time_to_init),
            trim: false,
            label: [DHCP configuration duration],
            side: "high",
        ),
    ),
)
