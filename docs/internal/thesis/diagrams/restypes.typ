#import "/packages.typ": *
#import "../lib.typ": *

#figure(
    label: <restypes>,
    caption: [Types de resources et leur propriétés],
    note: [
        Les différents types de resources avec qui peut les créer ou les
        modifier, qui peut les supprimer, et ou se situe le propriétaire dans le
        système.
    ],
    source: made-by-self,
    {
        show table.cell.where(x: 0).or(table.cell.where(y: 0)): set text(
            weight: "bold",
        )

        table(
            columns: (auto, 1fr, auto, 1fr),
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
