#import "../lib.typ": *

#let row-label(body) = {
    set par(justify: false)
    body
}

#let row(criterion: [], pertinence: "", faveur: [], justification: []) = {
    (
        row-label(criterion),
        row-label(pertinence),
        row-label(faveur),
        justification,
    )
}

#set page(flipped: true)
#figure(
    label: <scheduling>,
    caption: [Comparaison des modèles d'orchestration centralisés et
        décentralisés],
    source: made-by-self,
    {
        show table.cell.where(x: 0).or(table.cell.where(y: 0)): set text(
            weight: "bold",
        )

        table(
            columns: (3.7cm, auto, auto, 1fr),
            row-gutter: (2.2pt, auto),
            align: start,
            table.header[Critère][Pertinence][Faveur][Justification],
            ..row(
                criterion: [Contrôle global de la planification],
                pertinence: [Moyenne],
                faveur: [Centralisé],
                justification: [Une boucle unique peut optimiser la
                    réconciliation en tenant compte des dépendances et de la
                    hiérarchie des ressources.],
            ),
            ..row(
                criterion: [Flexibilité de planification],
                pertinence: [Moyenne],
                faveur: [Décentralisé],
                justification: [Chaque contrôleur peut régler son propre
                    intervalle d'exécution, son backoff et sa concurrence sans
                    coordination centrale.],
            ),
            ..row(
                criterion: [Réaction aux événements internes],
                pertinence: [Moyenne],
                faveur: [Centralisé],
                justification: [Déclencher une réconciliation sur changement
                    d'état est trivial dans une boucle centrale; dans un modèle
                    décentralisé, chaque contrôleur devrait gérer ses propres
                    souscriptions.],
            ),
            ..row(
                criterion: [Réaction aux événements externes],
                pertinence: [Moyenne],
                faveur: [Décentralisé],
                justification: [Les événements externes (ex. arrêt d'un
                    conteneur) peuvent être traités directement par le
                    contrôleur concerné. Dans le modèle centralisé, ces signaux
                    doivent transiter par l'orchestrateur.],
            ),
            ..row(
                criterion: [Détection d'un contrôleur bloqué],
                pertinence: [Faible],
                faveur: [Centralisé],
                justification: [L'orchestrateur central peut détecter un
                    contrôleur bloqué via un timeout sur l'appel de
                    réconciliation. Les contrôleurs décentralisés échouent
                    silencieusement.],
            ),
            ..row(
                criterion: [Assignation automatique du parent],
                pertinence: [Faible],
                faveur: [Centralisé],
                justification: [La réponse de `reconcile()` contient les
                    demandes de création, ce qui permet d'associer trivialement
                    la relation parent-enfant.],
            ),
            ..row(
                criterion: [Nombre d'appels API],
                pertinence: [Non retenu],
                faveur: [Centralisé],
                justification: [`reconcile()` regroupe l'état en un seul appel,
                    contre plusieurs requêtes par itération en mode
                    décentralisé. Non pertinent étant donné le faible nombre de
                    ressources et l'absence de contrainte de débit.],
            ),
            ..row(
                criterion: [Périmètre d'une panne],
                pertinence: [Non retenu],
                faveur: [Décentralisé],
                justification: [Une panne de l'orchestrateur central arrête
                    toutes les réconciliations. En mode décentralisé, la panne
                    est isolée au contrôleur concerné. Non pertinent car la
                    panne de n'importe quel contrôleur met en péril le système
                    entier.],
            ),
        )
    },
)
