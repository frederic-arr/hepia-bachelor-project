#import "../lib.typ": *

= Tests et validation
#todo-note[Tests et validation][
    - Scénarios de validation
    - Benchmarking:
        - empreinte mémoire; pour ceci, deux méthodes:
            - pour qqc de précis, utiliser l'allocateur Peak en Rust
            - de manière plus générale donner X MB de RAM à la VM, et chercher
                la plus grosse allocation qu'on puisse faire depuis un conteneur
        - time-to-boot
]
