import {useQuery} from "@tanstack/react-query";
import {useParams} from "react-router-dom";
import {invoke} from "@tauri-apps/api/core";

import {InputTodo} from "./InputTodo.jsx";
import { str2date } from "./str2date.jsx";

export default function PasteTodo() {
    const { id } = useParams();

    const { data: todo, isLoading, isError, error} = useQuery({
        queryKey: ['todo_item_'+id],
        queryFn: async () => invoke('get_todo_with_id', {id: Number(id)}),
    });

    const handleSendData = async (data) => {
        const res = {
            item: {
                title: data.title,
                work: data.work,
                start: str2date(data.start)?.toLocaleDateString(),
                end: str2date(data.end)?.toLocaleDateString(),
            }
        };
        await invoke("add_todo", res);
    };

    if (isLoading) {
        return ( <p> loading... </p> );
    }

    if (isError) {
        return ( <p> Error: {error} </p> );
    }

    const initForm = {
        title: todo.title,
        work: todo.work,
        start: todo.start_date?.replace(/-/g,"/"),
        end: todo.end_date==="9999-12-31" ? "" : todo.end_date.replace(/-/g,"/"),
    }

    return (
        <>
            <InputTodo send_data={handleSendData} init_val={initForm}/>
        </>
    );
}
