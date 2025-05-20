use url::Url;
#[derive(Debug)]

#[allow(dead_code)] // optional, therefore might this not be used
pub enum TaskFailureReason {
    TechnicalIssues,
    TaskIssues,
    ProblemSolving,
}

impl ToString for TaskFailureReason {
    fn to_string(&self) -> String {
        match self {
            TaskFailureReason::TechnicalIssues => "TechnicalIssues".to_string(),
            TaskFailureReason::TaskIssues => "TaskIssues".to_string(),
            TaskFailureReason::ProblemSolving => "ProblemSolving".to_string(),
        }
    }
}


#[allow(dead_code)] // in our current example we only report failure, so not used
pub fn report_success(minion_api : Url, minion_token : String, description: &str) {
    reqwest::blocking::Client::new()
        .post(minion_api.join("agent/task/complete").unwrap())
        .bearer_auth(minion_token)
        .header("Content-Type", "application/json")
        .body(format!("{{\"description\": \"{}\"}}", description))
        .send()
        .unwrap();
}

pub fn report_failure(minion_api : Url, minion_token : String, description: &str, reason: Option<TaskFailureReason>) {
    let body = match reason {
        Some(r) => format!(
            "{{\"reason\": \"{}\", \"description\": \"{}\"}}",
            r.to_string(),
            description
        ),
        None => format!("{{\"description\": \"{}\"}}", description),
    };

    reqwest::blocking::Client::new()
        .post(minion_api.join("agent/task/fail").unwrap())
        .bearer_auth(minion_token)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .unwrap();
}