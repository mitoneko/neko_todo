# 猫todo関係のすべてのmariadbオブジェクトの生成

create database if not exists neko_todo;

use neko_todo;

create table if not exists todo (
    id int unsigned auto_increment primary key,
    title varchar(128) not null,
    work varchar(2048),
    update_date date not null,
    start_date date,
    end_date date,
    done bool
    ) ;

create table if not exists tag (
    name varchar(128) primary key
    );

create table if not exists todo_tag (
    todo_id int unsigned references todo(id),
    tag_name varchar(128) references tag(name),
    primary key(todo_id, tag_name)
    );
