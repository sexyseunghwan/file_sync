use crate::common::*;



#[doc = "Functions that return the current UTC time -> NaiveDate"]
pub fn get_current_utc_naivedate() -> NaiveDate {
    
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.date_naive()
}


#[doc = "Functions that return the current UTC time -> NaiveDatetime"]
pub fn get_currnet_utc_naivedatetime() -> NaiveDateTime {

    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.naive_local()
}

 
#[doc = "Function that returns the current UTC time as a string"]
pub fn get_current_utc_naivedate_str(fmt: &str) -> Result<String, anyhow::Error> {

    let curr_time = get_current_utc_naivedate();
    let curr_time_str = get_str_from_naivedate(curr_time, fmt)?;

    Ok(curr_time_str)
}   


#[doc = "Function that returns the current UTC time as a string"]
pub fn get_current_utc_naivedatetime_str(fmt: &str) -> Result<String, anyhow::Error> {

    let curr_time = get_currnet_utc_naivedatetime();
    let curr_time_str = get_str_from_naivedatetime(curr_time, fmt)?;

    Ok(curr_time_str)
}   


#[doc = "Function that converts the date data 'naivedate' format to the string format"]
pub fn get_str_from_naivedatetime(naive_date: NaiveDateTime, fmt: &str) -> Result<String, anyhow::Error> {
    
    let result_date = naive_date.format(fmt).to_string();
    Ok(result_date)
}


#[doc = "Function that converts the date data 'naivedate' format to the string format"]
pub fn get_str_from_naivedate(naive_date: NaiveDate, fmt: &str) -> Result<String, anyhow::Error> {
    
    let result_date = naive_date.format(fmt).to_string();
    Ok(result_date)
}


#[doc = "Function that converts the date data 'naivedatetime' format to String format"]
pub fn get_str_from_naive_datetime(naive_datetime: NaiveDateTime, fmt: &str) -> Result<String, anyhow::Error> {
    
    let result_date = naive_datetime.format(fmt).to_string();
    Ok(result_date)
}


#[doc = "Function to change 'string' data format to 'NaiveDateTime' format"]
pub fn get_naive_datetime_from_str(date: &str, format: &str) -> Result<NaiveDateTime, anyhow::Error> {
    
    NaiveDateTime::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][get_naive_datetime_from_str()] Failed to parse date string: {:?} : {:?}", date, e)) 
}


#[doc = "Function to change 'string' data format to 'NaiveDate' format"]
pub fn get_naive_date_from_str(date: &str, format: &str) -> Result<NaiveDate, anyhow::Error> {
    
    NaiveDate::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][get_naive_date_from_str()] Failed to parse date string: {:?} : {:?}", date, e))
    
}


#[doc = "주어진 날짜 문자열이 유효한지 확인하고, UTC 기준으로 오늘과의 날짜 차이를 계산"]
/// # Arguments
/// `date_str` - 날짜를 나타내는 문자열
/// 
/// # Returns
/// * Result<i64, anyhow::Error>
pub fn calculate_date_difference_utc(date_str: &str) -> Result<i64, anyhow::Error> {
    
    /* 문자열을 NaiveDate 객체로 변환 */ 
    let parsed_date = NaiveDate::parse_from_str(date_str, "%Y%m%d")
        .map_err(|e| anyhow!("Invalid date format: {:?}", e))?;

    /* UTC 기준의 오늘 날짜 구하기 */ 
    let today = get_current_utc_naivedate();

    /* 날짜 차이 계산 */ 
    let duration = today.signed_duration_since(parsed_date);

    /* 날짜 차이의 일수 반환 */ 
    Ok(duration.num_days())
}
