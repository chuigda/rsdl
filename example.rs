type int = i64;

type float = f64;

type str = String;

/// 学生的信息
#[derive(Debug, Clone)]
pub struct StudentInfo {
    /// 学号
    pub id: int,
    /// 姓名
    pub name: str,
    /// 选课
    pub course: Vec<Course>,
    /// 成绩
    pub score: Vec<int>,
    #[cfg(feature = "thread_safety")]
    phantom: Phantom,
}

/// 课程的信息
#[derive(Debug, Clone)]
pub struct Course {
    /// 课程 ID
    pub id: int,
    /// 课程名称
    pub name: str,
    /// 学分
    pub credit: int,
    #[cfg(feature = "thread_safety")]
    phantom: Phantom,
}

type Phantom = PhantomData<*const ()>;
