use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum Student {
    Table,
    Id,
    /// 姓名
    Name,
    /// 学号
    StudentNo,
    /// 认定级别
    DifficultyLevel,
    /// 学院
    SecondarySchool,
    /// 班级
    Class,
    /// 性别
    Sex,
    /// "余额"
    Balance,
}

#[derive(DeriveIden)]
pub enum Item {
    Table,
    Id,
    /// 名称
    Name,
    // 规格
    Spec,

    // 进货单价
    Price,

    // 简单模式价格(一般苦难)
    PEasy,
    // 普通模式价格（困难）
    PNormal,
    // 困难模式价格（特别困难）
    PHard,

    // // 简单模式出售数量
    // SoldEasy,
    // // 普通模式出售数量
    // SoldNormal,
    // // 困难模式出售数量
    // SoldHard,

    // 积分价格
    PScore,
}

// #[derive(DeriveIden)]
// pub enum Record {
//     Table,
//     Id,
//     StudentId,
// }

// #[derive(DeriveIden)]
// pub enum RecordItem {
//     Table,
//     Id,
//     RecordId,
//     ItemId,
//     Quantity,
// }

#[derive(DeriveIden)]
/// 除了id和数量都是冗余字段
pub enum Record {
    Table,
    Id,
    StudentId,
    ItemId,

    /// 数量
    Quantity,

    /// 学号
    StudentNo,
    /// 认定级别
    StuDLevel,
    /// 折后价(单价)
    DiscountPrice,
    /// 原价(单价)
    OriginalPrice,
    /// 商品名称
    ItemName,
    /// 商品规格
    ItemSpec,
}
