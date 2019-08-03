use regex;
use std::io::Read;
use reqwest::{Client, IntoUrl, Response, Proxy, ClientBuilder};
use rustc_serialize::json;
use rustc_serialize::json::Json;
use std::ops::{Index, Add};
use std::time;
use chrono;

fn main() {
    let recent_changes_users = get_web("https://zh.wikipedia.org/w/api.php?action=query&format=json&list=recentchanges&rcprop=user&rcstart=2019-08-03T00%3A41%3A36%2E527Z&rcend=2019-08-03T00%3A11%3A36%2E527Z&rcshow=!bot|\
    !anon&rclimit=500");
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
    let mut i :usize = 0;
    for each in &users{
        if users.len() >= 45{
            let buffer = users[i..(i + 45)].to_vec();
            u_lists.push(buffer);
        }

        if i + 90 >= users.len(){
            u_lists.push(users[i+45..].to_vec());
            break;
        }else {
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
    println!("{:?}", users_info);
    println!("{:?}", users_info.len());
    println!();

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
    println!("数据整理完成，正在输出...");
    println!("活跃管理员列表：");
    for each in admin_list{
        println!("{}", each);
    }
    println!("活跃回退员列表：");
    for each in r_list{
        println!("{}", each);
    }
    println!("活跃巡查员列表：");
    for each in p_list{
        println!("{}", each);
    }

}

fn get_web(url: &str) -> String{
    let mut html = reqwest::get(url).unwrap();
    let mut string = String::new();
    html.read_to_string(& mut string);
    string
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