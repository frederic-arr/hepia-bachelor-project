#import "../lib.typ": *

#show heading.where(level: 3): set heading(outlined: false)

= Présentation du système

#todo-note[
    - Présente le système pour un utilisateur.
    - Mélange un petit peu un "Quick Start" / "Getting Started" et un "Learn
        More" de haut niveau
    - Volume estimé: ~10-13 vrai pages
]

#todo-inline[Amorce du chapitre + plan interne + synthèse brève.]

#todo-inline[Référencer les divers outils.]

== Principes

#[
    #show heading.where(level: 3): set heading(outlined: false)

    // TODO: référencer le projet de semestre
    Le système s'articule autour de cinq principes fondamentaux: la
    déclarativité, l'automatisation, la légèreté, la sécurité par défaut et la
    portabilité. Ces principes ont été dégagés de l'analyse des besoins
    présentés dans le projet de semestre, et répondent à un contexte de
    déploiement particulier: des environnements hétérogènes, non distribués,
    parfois sur du matériel aux ressources contraintes et administrés par un
    utilisateur seul.

    === Déclarativité

    Tout d'abord, plutôt que de décrire les étapes à exécuter pour atteindre un
    état donné, l'administrateur décrit directement l'état final souhaité. Le
    système détermine lui-même les actions nécessaires pour y parvenir, quelle
    que soit l'état courant de la machine.

    // TODO: Besoin de dire pourquoi ça facilite le retour arrière?
    Ce choix est particulièrement adapté à un contexte de reconfiguration
    fréquente sur des environnements variés. Une approche impérative obligerait
    l'administrateur à prendre en compte l'état courant avant chaque opération
    et rend plus difficile la récupération en cas d'erreur durant la
    configuration du système. La déclarativité permet d'améliorer la
    reproductibilité des déploiements et simplifie le retour à un état antérieur
    en cas d'erreur humaine.

    === Automatisation

    Toute opération administrative, depuis l'installation initiale jusqu'à la
    reconfiguration, doit pouvoir être réalisée sans intervention manuelle.
    L'objectif recherché est de permettre une intégration simple dans le
    processus de déploiement, en particulier lorsque celui-ci repose sur une
    approche GitOps/DevOps.

    Les interventions manuelles pour le diagnostic ou la récupération restent
    possibles, mais doivent demeurer exceptionnelles et clairement distinctes du
    mode normal d'exploitation.

    === Légèreté

    Le système doit présenter une empreinte mémoire minimale afin de laisser un
    maximum de ressources aux services déployés par l'administrateur. De fait,
    il n'assume qu'un seul rôle: servir de plateforme d'exécution pour des
    services conteneurisés et aucun composant superflu n'est présent.

    Ce principe se justifie par des déploiement sur des appareils basse
    consommation disposant de peu de resources. Cela présente en outre deux
    effets secondaires bénéfiques: une configuration plus simple en raison du
    petit nombre de composants et une surface d'attaque plus réduite.

    === Sécurité par défaut

    Le système adopte des paramètres restrictifs dès l'installation, sans
    configuration supplémentaire. Les services internes et ceux déployés par
    l'administrateur sont isolés aussi fortement que possible, afin de limiter
    les risques de mouvement latéral en cas de compromission d'un composant.

    L'analyse des besoins révèle principalement un usage pour l’hébergement de
    services accessible via Internet et donc soumis en permanence à des
    tentatives d'intrusion. Des mécanismes d'exception strictement contrôlés
    restent disponibles pour les cas qui le requièrent, notamment lorsque l'hôte
    assume un rôle de routeur ou de pare-feu réseau.

    === Portabilité

    L'interface d'administration et le format de configuration doivent rester
    identiques quelle que soit la configuration matérielle ou réseau
    sous-jacente. En dehors des dépendances introduites explicitement par
    l'administrateur, le transfert du système d'un environnement à un autre ne
    doit nécessiter aucune adaptation de la configuration existante.

    Dans un contexte où un même administrateur administre plusieurs machines aux
    profils distincts, cette homogénéité réduit la charge cognitive et rend les
    procédures opérationnelles transférables sans friction.
]

== Ressources

=== Définition <simple-res-def>
L'administration et la configuration du système est organisé autour des
ressources. Une ressource est simplement l'abstraction d'un ou plusieurs objets
concrets au sein du système. Par exemple, la ressource "conteneur" représente le
concept du même nom; la ressource "interface réseau" représente quant à elle un
lien réseau, une ou plusieurs routes, et une ou plusieurs addresses.

=== Réseau
#todo[Firewall, VPN, DHCP, IP statique, IPv6, etc.]

=== Conteneurisation
#todo[Conteneur, gestion des images, registres privé, Podman, Secrets, Limites]

=== Stockage et volumes
#todo[
    Partitionnement disque, volume, cache, data, chiffrement (on peut combiner
    plusieurs methodes de chiffrement!)
]

=== Portabilité de la configuration
#todo[Portabilité][
    - On peut matcher une interface par une MAC address (partielle), nom, index,
        etc.
    - Idem pour les disques
    - En tous les cas, si une config a trois interface réseau mais que
        physiquement il y en a que une sur la nouvelle machine, pas de magie
]

== Administration du système
=== Exigences matérielles
#todo[Hardware requirements][
    - RAM
    - Storage
]

=== Interface d'administration
#todo[Interface d'administration][
    - Parler de la CLI avec un tableau des commandes les plus importantes
    - Parler de Terraform vite fait
    - Parler de comment on s'authentifie a l'interface (token, mTLS, etc.)
    - Parler des commandes importantes
        - Push config
        - Voir l'état
        - Téléverser des fichiers
        - Diagnostic réseau
        - exec dans un conteneurs
        - les logs
        - reboot
]

=== Sources de configuration
#todo[Config src][
    - disk
    - remote
    - cloud-init
    - etc.
    - Schéma des sources
]

=== Monitoring et observabilité
#todo[Monitoring][
    - En gros logs accessible via le CLI, ou exporté via les trucs standard
        (Loki, ES, etc.)
    - Pour l'observabilité, idem + exporté via Prometheus
    - Export des métriques et logs host *ET* conteneurs
]

=== Persistence des données
#todo[Persistence des données][
    - Comment consulter le stockage
    - Parler du fait que le disque est auto-déverrouillé + risques que cela
        implique
    - Parler en particulier du risque du mode "passphrase" pour le chiffrement
    - On peut aussi utiliser une passphrase en mode manuel, dans ce cas le
        système n'est pas auto-déverrouillé
]

== Cycle de vie du système
=== Personnalisation de l'image
#todo[Personnalisation][
    - Comment intégrer de nouvelles options du noyau
    - De nouveau drivers
    - Pre-bake une image conteneur ou une config
    - etc
]
=== Méthodes de démarrage
#todo[Boot][
    - boot en réseau
    - mode in-memory
    - différent bootloader
]

=== Installation initiale
#todo[
    Parler du fait qu'il suffit simplement de push une configuration valide sur
    un système vierge en maintenance.
]

=== Mise à jour et retours en arrière
#todo[
    - Parler du mécanisme de mise à jour du système avec le champ `osVersion` et
        l'installation A/B.
    - Parler du fait qu'on peut rollback la config ou la version du système (ou
        les deux).
]

=== Maintenance
#todo[
    Parler du mode de maintenance et du mode d'authentication additionnel propre
    à celui-ci.
]

=== Sauvegarde et restauration
#todo[
    - Parler de comment backup/restore le système
    - Comment exporter un volume/conteneur individuellement aussi
]

== Exemple d'utilisation
#todo[Deployer un conteneur simple avec un serveur HTTP]
