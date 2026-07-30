#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq
#import "../data/time-noinstall.typ": *

#let lim = calc.max(..time_to_run_container)
#let lim = calc.ceil(lim / 1000) * 1000

#figure(
    label: <val-container-time-noinstall>,
    caption: [
        Temps de démarrage d'un conteneur
    ],
    note: [
        Temps entre le la soumission d'une configuration créant un conteneur, le
        téléchargement de l'image et la réception d'une requête sur un port
        arbitraire de l'hôte faite par le conteneur. \
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
            delta(time_to_downloading_image, time_to_kernel),
            trim: false,
            label: [Time until image downloading],
            side: "low",
        ),
        lq.hviolin(
            delta(time_to_run_container, time_to_kernel),
            trim: false,
            label: [Time until container started],
            side: "low",
        ),
        lq.hviolin(
            delta(time_to_run_container, time_to_downloading_image),
            trim: false,
            label: [Image download duration],
            side: "high",
        ),
    ),
)
