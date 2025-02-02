import { useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Grid, GridItem, HStack, IconButton, Switch, Text} from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { AiOutlineFileAdd } from "react-icons/ai";
import { useState } from "react";
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

    const queryClient = useQueryClient();
    const [ IsIncomplete, setIsIncomplete ] = useState(true);
    const onIsIncompleteChange = async (e) => {
        setIsIncomplete(e.target.checked);
        await invoke("set_is_incomplete", {isIncomplete: e.target.checked});
        queryClient.invalidateQueries({ queryKey: ['todo_list']});
    }; 


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
                <Switch checked={IsIncomplete} onChange={onIsIncompleteChange}>
                    未完了のみ
                </Switch>
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
            <Text> すべて表示スイッチの状態は、{IsIncomplete ? "On" : "Off"} です。</Text>
        </>
    );
}


export default TodoList;
