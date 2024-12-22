import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Grid, GridItem} from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import TodoItem from "./todoitem";

const get_todo_list = async () => invoke('get_todo_list') ;

function TodoList() {

    const { data: todos, isLoading: isTodoListLoading , isError, error} = useQuery({
        queryKey: ['get_todo_list'],
        queryFn: get_todo_list,
    });

    const navi = useNavigate();
    const handleClick = () => navi('/login');

    if (isTodoListLoading) {
        return ( <p> loading... </p>);
    }

    if (isError) {
        return ( <p> エラーだよ。{error}</p> );
    }

    console.log(todos);
    return (
        <>
            <button type="button" onClick={handleClick}> 入力欄への遷移 </button>
            <h1>テスト</h1>
            <Grid templateColumns="repeat(4, 1fr)" gap="md">
                {todos?.map( todo_item => {
                    return (
                        <GridItem key={todo_item.title} w="full" rounded="md" bg="primary">
                            <TodoItem item={todo_item}/>
                        </GridItem>
                    )}
                )}
            </Grid>
        </>
    );
}


export default TodoList;
