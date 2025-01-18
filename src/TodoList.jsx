import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Grid, GridItem, HStack, IconButton} from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { AiOutlineFileAdd } from "react-icons/ai";
import "./App.css";
import TodoItem from "./todoitem";

const get_todo_list = async () => invoke('get_todo_list') ;

function TodoList() {

    const { data: todos, isLoading: isTodoListLoading , isError, error} = useQuery({
        queryKey: ['todo_list'],
        queryFn: get_todo_list,
    });

    const navi = useNavigate();
    const handleAddTodo = () => navi('/addtodo');

    if (isTodoListLoading) {
        return ( <p> loading... </p>);
    }

    if (isError) {
        return ( <p> エラーだよ。{error}</p> );
    }

    console.log(todos);
    return (
        <>
            <HStack>
                <IconButton icon={<AiOutlineFileAdd/>} onClick={handleAddTodo}/>
            </HStack>

            <h1>現在の予定</h1>
            <Grid templateColumns="repeat(4, 1fr)" gap="md">
                {todos?.map( todo_item => {
                    return (
                        <GridItem key={todo_item.id} w="full" rounded="md" bg="primary">
                            <TodoItem item={todo_item}/>
                        </GridItem>
                    )}
                )}
            </Grid>
        </>
    );
}


export default TodoList;
