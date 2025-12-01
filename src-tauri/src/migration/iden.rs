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
    /// 专业
    Major,
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
    // 原价
    Price,
    // 7折(一般困难)
    PEasy,
    // 5折（困难）
    PNormal,
    // 3折（特别困难）
    PHard,
    // 积分
    PScore,
}

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
