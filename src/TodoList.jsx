import { useQuery, } from "@tanstack/react-query";
import { Container, Grid, GridItem, } from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import TodoItem from "./todoitem";
import TodoItemToolbar from "./TodoListToolbar.jsx";

const get_todo_list = async () => invoke('get_todo_list') ;

function TodoList() {

    const { data: todos, isLoading: isTodoListLoading , isError, error} = useQuery({
        queryKey: ['todo_list'],
        queryFn: get_todo_list,
    });

    if (isTodoListLoading) {
        return ( <p> loading... </p>);
    }

    if (isError) {
        return ( <p> エラーだよ。{error}</p> );
    }

    console.log(todos);
    return (
        <>
            <Container gap="0" bg="backgound">
                <TodoItemToolbar/>

                <h1>現在の予定</h1>
                <Grid templateColumns="repeat(4, 1fr)" gap="md" >
                    {todos?.map( todo_item => {
                        return (
                            <GridItem key={todo_item.id} w="full" rounded="md" bg="primary">
                                <TodoItem item={todo_item}/>
                            </GridItem>
                        )}
                    )}
                </Grid>
            </Container>
        </>
    );
}


export default TodoList;
