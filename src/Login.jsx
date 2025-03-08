/* ログイン画面 */

import { useForm } from "react-hook-form";
import { VStack, FormControl, Input, Button, Text, Container, PasswordInput, useAsyncCallback, Heading } from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { Link, useNavigate } from "react-router-dom";
import { useState } from "react";
import {useQueryClient} from "@tanstack/react-query";

function Login() {
    const { register, handleSubmit, formState: {errors} } = useForm();
    const [ sendMessage, setSendMessage ] = useState('');
    const navi = useNavigate();
    const queryClient = useQueryClient();
    
    const [isSending, onSubmit] = useAsyncCallback( async (data) => {
        try {
            await invoke('login', { name: data.name, password: data.pass });
            queryClient.invalidateQueries("check_login");
            navi('/');
        } catch (e) {
            if (e === "WrongPassword") {
                setSendMessage("ユーザー名、または、パスワードが違います。");
            } else if (e === "NotFoundUser") {
                setSendMessage("ユーザー名、または、パスワードが違います。");
            } else {
                setSendMessage('エラーが発生しました。{' + e + '}');
                console.log(e);
            }
        }
    },[]);

    return (
        <>
            <Container>
                <Link to="/regist_user">
                    <Text textDecorationLine="underLine" textAlign="right">
                        新規ユーザー登録
                    </Text>
                </Link>
                <Heading> ログイン </Heading>
                <VStack as="form" onSubmit={handleSubmit(onSubmit)}>
                    <FormControl 
                        invalid={!!errors.name} 
                        label="ユーザー名" 
                        errorMessage={errors?.name?.message} 
                    >
                        <Input {...register("name", {required: "入力は必須です。"},)}/>
                    </FormControl>
                    <FormControl
                        invalid={!!errors.pass}
                        label="パスワード"
                        errorMessage={errors?.pass?.message}
                    >
                        <PasswordInput {...register("pass", {required: "入力は必須です。"},)}/>
                    </FormControl>
                    <Button type="submit" w="30%" ml="auto" mr="auto" 
                        loading={isSending} loadingText="処理中" > 
                        ログイン 
                    </Button>
                    <Text>{sendMessage}</Text>
                </VStack>
            </Container>
        </>
    );
}

export default Login;

