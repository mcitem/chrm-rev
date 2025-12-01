use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Student::Table)
                    .if_not_exists()
                    .col(integer(Student::Id).primary_key())
                    .col(string(Student::Name))
                    .col(string(Student::StudentNo))
                    .col(string(Student::DifficultyLevel))
                    .col(string_null(Student::SecondarySchool))
                    .col(string_null(Student::Class))
                    .col(string_null(Student::Sex))
                    .col(string_null(Student::Major))
                    .col(decimal_len(Student::Balance, 10, 2))
                    .take(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Item::Table)
                    .if_not_exists()
                    .col(integer(Item::Id).primary_key())
                    .col(string(Item::Name))
                    .col(string(Item::Spec))
                    .col(decimal_len(Item::Price, 10, 2))
                    .col(decimal_len(Item::PEasy, 10, 2))
                    .col(decimal_len(Item::PNormal, 10, 2))
                    .col(decimal_len(Item::PHard, 10, 2))
                    .col(decimal_len(Item::PScore, 10, 2))
                    .take(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Record::Table)
                    .if_not_exists()
                    .col(integer(Record::Id).primary_key())
                    .col(integer(Record::StudentId))
                    .col(integer(Record::ItemId))
                    .col(string(Record::StudentNo))
                    .col(string(Record::StuDLevel))
                    .col(integer(Record::Quantity))
                    .col(decimal_len(Record::DiscountPrice, 10, 2))
                    .col(decimal_len(Record::OriginalPrice, 10, 2))
                    .col(string(Record::ItemName))
                    .col(string(Record::ItemSpec))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-record-student-id")
                            .from(Record::Table, Record::StudentId)
                            .to(Student::Table, Student::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-record-item-id")
                            .from(Record::Table, Record::ItemId)
                            .to(Item::Table, Item::Id),
                    )
                    .take(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Record::Table).if_exists().take())
            .await?;
        manager
            .drop_table(Table::drop().table(Item::Table).if_exists().take())
            .await?;
        manager
            .drop_table(Table::drop().table(Student::Table).if_exists().take())
            .await?;
        Ok(())
    }
}
