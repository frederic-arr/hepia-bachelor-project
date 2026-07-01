#import "/packages.typ": *
#import "../lib.typ": *

#figure(
    label: <restypes>,
    caption: [Types de resources et leur propriétés],
    note: [
        Montre quel acteur une ressource peut-elle être crée, qu'elle lien
        aura-t-elle avec son créateur, et qui en sera le détenteur, autrement
        dit, qui peut la supprimer ou la modifier.
    ],
    source: made-by-self,
    {
        show table.cell.where(x: 0).or(table.cell.where(y: 0)): set text(
            weight: "bold",
        )

        table(
            columns: (auto, auto, auto, 1fr),
            rows: (auto, 1.5em, 1.5em, 1.5em),
            align: center + horizon,
            table.header(
                [Type],
                [Est créé par],
                [Est détenu par],
                [Lien de dépendance avec le créateur],
            ),
            ..(
                [Statique],
                table.cell(colspan: 2)[L'administrateur],
                table.cell(rowspan: 2)[Aucun],
            ),
            ..(
                [Dynamique],
                [Une autre ressource],
                [Le créateur],
            ),
            ..(
                [Mutualisé],
                [Une ou plusieurs ressources],
                [L'orchestrateur],
                [Est une dépendance],
            ),
        )
    },
)
