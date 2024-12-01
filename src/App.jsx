import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Grid, GridItem} from "@yamada-ui/react";
//import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import TodoItem from "./todoitem";

const get_todo_list = async () => invoke('get_todo_list') ;

function App() {

    const { data: todos, isLoading: isTodoListLoading } = useQuery({
        queryKey: ['get_todo_list'],
        queryFn: get_todo_list,
    });

    if (isTodoListLoading) {
        return ( <p> loading... </p>);
    }

    console.log(todos);
    return (
        <>
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


export default App;
