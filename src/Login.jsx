/* ログイン画面 */

import { useForm } from "react-hook-form";
import { VStack, FormControl, Input, Button, Text } from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { Link, useNavigate } from "react-router-dom";
import { useState } from "react";
import {useQueryClient} from "@tanstack/react-query";

function Login() {
    const { register, handleSubmit, formState: {errors} } = useForm();
    const [ sendMessage, setSendMessage ] = useState('');
    const navi = useNavigate();
    const queryClient = useQueryClient();
    
    const onSubmit = async (data) => {
        try {
            setSendMessage('処理中です。');
            await invoke('login', { name: data.name, password: data.pass });
            queryClient.invalidateQueries("check_login");
            navi('/');
        } catch (e) {
            setSendMessage('エラーが発生しました。{' + e + '}');
            console.log(e);
        }
    };

    return (
        <>
            <Link to="/regist_user">新規ユーザー登録</Link>
            <h1> ログイン </h1>
            <VStack as="form" onSubmit={handleSubmit(onSubmit)}>
                <FormControl 
                    isInvalid={!!errors.name} 
                    label="ユーザー名" 
                    errorMessage={errors?.name?.message} 
                >
                    <Input {...register("name", {required: "入力は必須です。"},)}/>
                </FormControl>
                <FormControl
                    isInvalid={!!errors.pass}
                    label="パスワード"
                    errorMessage={errors?.pass?.message}
                >
                    <Input {...register("pass", {required: "入力は必須です。"},)}/>
                </FormControl>
                <Button type="submit" w="30%" ml="auto" mr="auto"> ログイン </Button>
                <Text>{sendMessage}</Text>
            </VStack>
        </>
    );
}

export default Login;

