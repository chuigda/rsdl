/// 学生的信息
#[derive(Debug, Clone)]
pub struct StudentInfo {
    /// 学号
    pub id: i64,
    /// 姓名
    pub name: String,
    /// 选课
    pub course: Vec<Course>,
    /// 成绩
    pub score: Vec<i64>,
    #[cfg(feature = "thread_safety")]
    phantom: Phantom,
}

/// 课程的信息
#[derive(Debug, Clone)]
pub struct Course {
    /// 课程 ID
    pub id: i64,
    /// 课程名称
    pub name: Box<String>,
    /// 学分
    pub credit: i64,
    #[cfg(feature = "thread_safety")]
    phantom: Phantom,
}

type Phantom = PhantomData<*const ()>;
