use std::io::Read;
use rustc_serialize::json::Json;
use std::{time, io};
use chrono;
use chrono::prelude::*;
use std::thread;
use std::fs::File;

fn main(){
    const EACH_TIME:u64  = 600;
    const CHECK_TIMES:u32  = 5;
    loop {
        let mut count = 0;
        let mut results: Vec<CheckResult> = Vec::new();
        let time = Utc::now().naive_utc().to_string();
        loop {
            let result = run_check();
            println!("[{}]数据整理完成，正在输出...", time);
            println!("活跃管理员列表[{}人在线]：", result.admins_num);
            for each in &result.admins {
                println!("\t{}", each);
            }
            println!("活跃回退员列表[{}人在线]：", result.rollbackers_num);
            for each in &result.rollbackers {
                println!("\t{}", each);
            }
            println!("活跃巡查员列表[{}人在线]：", result.patroller_num);
            for each in &result.patroller {
                println!("\t{}", each);
            }
            results.push(result);
            count += 1;
            if count >= CHECK_TIMES { break; }
            thread::sleep(time::Duration::from_secs(EACH_TIME));
        }
        let mut a_number = 999999999;
        let mut min_a_time = String::new();
        let mut r_number = 999999999;
        let mut min_r_time = String::new();
        let mut p_number = 999999999;
        let mut min_p_time = String::new();

        let mut admin_c: Vec<UserCount> = Vec::new();
        let mut rollbacker_c: Vec<UserCount> = Vec::new();
        let mut patroller_c: Vec<UserCount> = Vec::new();
        for each in &results {
            if each.admins_num > a_number {
                a_number = each.admins_num;
                min_a_time = (&each.time).to_string()
            }
            if each.rollbackers_num > r_number {
                r_number = each.rollbackers_num;
                min_r_time = (&each.time).to_string()
            }
            if each.patroller_num > p_number {
                p_number = each.patroller_num;
                min_p_time = (&each.time).to_string()
            }
        }



        for each in &mut results {
            for eac in &each.admins {
                let mut inside: bool = false;
                'a: for ea in &mut admin_c {
                    if ea.user_name == eac.to_string() {
                        ea.edit_count += 1;
                        inside = true;
                        break 'a;
                    }
                }
                if !inside {
                    admin_c.push(UserCount::new(eac.to_string()));
                }
            }

            for eac in &each.rollbackers {
                let mut inside = false;
                'b: for ea in &mut rollbacker_c {
                    if ea.user_name == eac.to_string() {
                        ea.edit_count += 1;
                        inside = true;
                        break 'b;
                    }
                }
                if !inside {
                    rollbacker_c.push(UserCount::new(eac.to_string()));
                }
            }

            for eac in &each.patroller {
                let mut inside = false;
                'c: for ea in &mut patroller_c {
                    if ea.user_name == eac.to_string() {
                        ea.edit_count += 1;
                        inside = true;
                        break 'c;
                    }
                }
                if !inside {
                    patroller_c.push(UserCount::new(eac.to_string()));
                }
            }
        }

        // 计算出最活跃的管理人员
        let mut et = 0;
        let mut a_max_name: Vec<String> = count_user_edit(& mut admin_c, CHECK_TIMES);
        let mut r_max_name: Vec<String> = count_user_edit(& mut rollbacker_c, CHECK_TIMES);
        let mut p_max_name: Vec<String> = count_user_edit(& mut patroller_c, CHECK_TIMES);

        //计算出各个管理人员的活跃指数
        //指数计算方法：按照每天24小时中有15个小时可以工作，再以检查结果的总出现次数计算。

        let mut out_message = "\n一轮回的巡查已经结束，即将输出结果：\n".to_string();
        out_message = out_message + "各个管理员的活跃程度：";
        out_message = out_message + "管理员：\n";
        out_message = out_message + format_active(admin_c).as_str();
        out_message = out_message + "回退员：\n";
        out_message = out_message + format_active(rollbacker_c).as_str();
        out_message = out_message + "巡查员：\n";
        out_message = out_message + format_active(patroller_c).as_str();
        out_message = out_message + "\n最活跃的管理员：\n";
        for each in a_max_name {
            out_message = "\t\t".to_string() + out_message.as_str() + each.as_str() + "\n";
        }
        out_message = out_message + "最活跃的回退员：\n";
        for each in r_max_name {
            out_message = "\t\t".to_string() + out_message.as_str() + each.as_str() + "\n";
        }
        out_message = out_message + "最活跃的巡查员：\n";
        for each in p_max_name {
            out_message = "\t\t\t".to_string() + out_message.as_str() + each.as_str() + "\n";
        }
        println!();
        println!("{}", out_message);
//        let mut f = File::create("./message.txt").unwrap();
//        io::copy(&mut out_message, &mut f);
        //todo 改用文件写入
    }
}


fn run_check() -> CheckResult {
    let now_time = NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0).to_string().replace(" ", "T") + ".692Z";
    let last_time = NaiveDateTime::from_timestamp(Utc::now().timestamp()-600, 0).to_string().replace(" ", "T") + ".692Z";
    let recent_changes_users = get_web(("https://zh.wikipedia.org/w/api.php?action=query&format=json&list=recentchanges&rcprop=user&rcstart=".to_string() + now_time.as_str() + "&rcend=" + last_time.as_str() + "&rcshow=!bot|\
    !anon&rclimit=500").as_str());
    let js = Json::from_str(&recent_changes_users).unwrap();
    let obj = js.as_object();
    let result_obj = obj.unwrap().get("query").unwrap().as_object().unwrap().get("recentchanges");
    let mut users : Vec<String> = Vec::new();
    let mut u_lists:Vec<Vec<String>> = Vec::new();
    let mut users_info:Vec<UserInfo> = Vec::new();

    for each in result_obj.unwrap().as_array() {
        for each in each {
            let user = each.as_object().unwrap().get("user").unwrap().as_string().unwrap();
            if !users.contains(&user.to_string()){
                users.push(user.to_string());
            }
        }
    }
//    println!("{:?}", users.len());
    println!("{}", &users.len());
    /**
    *变量users：通过匹配得到的用户列表
    *变量u_list：为了符合API的需求，我们必须将用户名分为45个一组，然后将每个组压入u_list这个矢量（数组）中
    **/
    let mut i :usize = 0; //表示当前users的初始下标，初始为0，随后会增加以进行数组分割
    for each in &users{ //for循环，每次循环获取矢量（其他语言称数组）users中的一个元素。
        if users.len() <= 45 as usize {  //如果users的长度小于45，则直接将整个数组压入u_list中并跳出循环
            u_lists.push(users);
            break;
        }

        if users.len() >= 45{ //如果users的长度大于45，则进行进一步操作：
            let buffer = users[i..(i + 45)].to_vec(); //创建一个缓冲区，将users的第i位（见上定义i之处）到(i + 45)位储存起来。
            u_lists.push(buffer); //将缓冲区（相当于一个组）压入矢量中（该矢量作用见上注释）
        }else if i + 90 >= users.len(){ //如果发现进行下一次切割会导致下标越位（也就是切到头了），则进行最后一次切割
            u_lists.push(users[i + 45..].to_vec()); //将末尾值压入矢量中
        }else { //如果下一次分割还没分割到尾，则继续循环往复分割。
            i += 45;
        }
    }

    for each in u_lists{
        let mut get_url = String::new();
        for each in each{
            get_url = get_url + (each + "|").as_str();
        }
        let result_ = get_web(("https://zh.wikipedia.org/w/api.php?action=query&format=json&list=users&usprop=groups&ususers=".to_string() + get_url.as_str()).as_str());
        let json_info = Json::from_str(result_.as_str()).unwrap();
        let mut count = 0;
        for each in json_info.as_object().unwrap().get("query").unwrap().as_object().unwrap().get("users").unwrap().as_array().unwrap(){
            count += 1;
            if count == 1{
                continue;
            }
            let obj = each.as_object().unwrap();
            let mut group = Group::None;
            'a : for each in obj.get("groups").unwrap().as_array().unwrap(){
                let mut g = String::new();
                if each.as_string() == None{
                    println!("WARNING:FOUND NONE VALUE --{:?}", each);
                    break 'a;
                }
                if each.as_string().unwrap() == "sysop".to_string(){
                    group = Group::Admin;
                    break 'a;
                }
                if each.as_string().unwrap() == "rollbacker".to_string(){
                    if group == Group::Patroller{
                        group = Group::RP;
                        break 'a;
                    }
                    group = Group::Rollbacker
                }
                if each.as_string().unwrap() == "patroller".to_string(){
                    if group == Group::Rollbacker{
                        group = Group::RP;
                        break 'a;
                    }
                    group = Group::Patroller;
                }
            }
            if group != Group::None {
                users_info.push(UserInfo::new(obj.get("name").unwrap().as_string().unwrap().to_string(), group));
            }
        }
    }


    let mut admin_list:Vec<String> = Vec::new();
    let mut r_list:Vec<String> = Vec::new();
    let mut p_list:Vec<String> = Vec::new();

    for each in users_info{
        match each.group {
            Group::Admin => admin_list.push(each.user),
            Group::Rollbacker => r_list.push(each.user),
            Group::Patroller => p_list.push(each.user),
            Group::RP =>{r_list.push(each.user.clone()); p_list.push(each.user);}
            Group::None => {}
        }
    }

    let mut out_put = CheckResult::new();
    out_put.time = last_time + now_time.as_str();
    out_put.admins_num = admin_list.len();
    out_put.rollbackers_num = r_list.len();
    out_put.patroller_num = p_list.len();
    for each in admin_list{
        out_put.admins.push(each);
    }
    for each in r_list{
        out_put.rollbackers.push(each);
    }
    for each in p_list{
        out_put.patroller.push(each);
    }
    out_put
}

fn get_web(url: &str) -> String{
    let mut html = reqwest::get(url).unwrap();
    let mut string = String::new();
    html.read_to_string(& mut string);
    string
}

fn count_user_edit(count_vec: & mut Vec<UserCount>, check_times:u32) -> Vec<String>{
    let mut max_name:Vec<String> = Vec::new();
    let mut et = 0;
    for each in count_vec {
        if each.edit_count >= et {
            et = each.edit_count;
            max_name.push(each.user_name.clone());
        }
        each.active_index = (each.edit_count as f32/check_times as f32)*100.0;
    }
    max_name
}

fn format_active(user_counts:Vec<UserCount>) -> String{
    let mut message = String::new();
    for each in user_counts{
        message = message + each.user_name.as_str() + ":" + each.active_index.to_string().as_str() + "%\n";
    }
    message
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct UserInfo{
    user : String,
    group : Group,
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Group{
    Admin,
    Rollbacker,
    Patroller,
    RP,
    None,
}

impl UserInfo{
    fn new(name: String, group: Group) -> UserInfo{
        UserInfo{user:name, group}
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct CheckResult{
    time:String,
    admins:Vec<String>,
    rollbackers:Vec<String>,
    patroller:Vec<String>,
    admins_num:usize,
    rollbackers_num:usize,
    patroller_num:usize,
}

impl CheckResult{
    fn new() -> CheckResult{
        CheckResult{time:String::new(),admins:Vec::new(), rollbackers:Vec::new(), patroller:Vec::new(), admins_num:0, rollbackers_num:0, patroller_num:0}
    }
}

#[derive(Debug, PartialEq, Clone)]
struct UserCount {
    user_name: String,
    edit_count : u32,
    active_index : f32,
}

impl UserCount{
    fn new(name:String) -> UserCount{
        UserCount{user_name:name, edit_count:1, active_index:0.0}
    }
}