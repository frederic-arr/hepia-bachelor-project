#import "@preview/glossarium:0.5.10": (
    gls, glspl, make-glossary, print-glossary, register-glossary,
)
#show: make-glossary
#let entry-list = (
    (
        key: "mutex",
        short: "Mutex",
        description: [verrou d'exclusion mutuel],
    ),
    (
        key: "mib",
        short: "MiB, GiB",
        long: "Mébibyte, Gibibyte",
        description: [unité de mesure en puissance de deux (1 MiB = 1024 \* 1024
            octets)],
    ),
    (
        key: "namespace",
        short: "Namespace",
        description: [mécanisme du noyau Linux permettant d'isoler un processus
            d'une ressource système donnée (réseau, système de fichiers,
            identifiants de processus, etc.), de sorte que ce processus perçoive
            une instance propre de cette ressource],
    ),
    (
        key: "capability",
        short: "Capability",
        description: [unité de privilège Linux, plus fine que la distinction
            binaire root/non-root, permettant d'accorder à un processus un
            sous-ensemble précis des privilèges normalement réservés à root (par
            exemple `CAP_NET_ADMIN` pour l'administration réseau)],
    ),
    (
        key: "rootless",
        short: "Rootless",
        description: [mode d'exécution dans lequel un processus, tel qu'un
            runtime de conteneurs, fonctionne sans privilège root, réduisant
            ainsi l'impact d'une éventuelle compromission],
    ),
    (
        key: "cgroup",
        short: "Cgroup",
        long: "Control Group",
        description: [mécanisme du noyau Linux permettant de limiter,
            comptabiliser et isoler l'usage de ressources (CPU, mémoire, etc.)
            d'un ensemble de processus],
    ),
    (
        key: "terraform",
        short: "Terraform",
        description: [outil d'infrastructure-as-code permettant de décrire, au
            moyen de fichiers déclaratifs, l'état souhaité d'une infrastructure,
            puis de le réconcilier via des fournisseurs (_providers_) dédiés à
            chaque plateforme ou système ciblé],
    ),
    (
        key: "tpm",
        short: [TPM 2.0],
        long: [_Trusted Platform Module 2.0_],
        description: [
            Un TPM 2.0 est un composant de sécurité, typiquement matériel,
            implémentant la deuxième édition de la norme ISO/IEC 11889. Il
            permet de générer, de stocker et d'utiliser des clefs
            cryptographiques de manière protégée, et de mesurer l'état du
            système lors du démarrage @bib-intelTpm. La version 2.0 étend les
            algorithmes cryptographiques disponibles et introduit diverses
            améliorations par rapport aux versions précédentes.
        ],
    ),
)
#register-glossary(entry-list)

= Glossaire
#print-glossary(
    entry-list,
    show-all: true,
)
