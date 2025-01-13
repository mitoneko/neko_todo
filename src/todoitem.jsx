// todoリストの各アイテム
import { SimpleGrid, GridItem } from "@yamada-ui/react";

export default function TodoItem({item}) {
    let end_date = new Date(item.end_date); 
    if (item.end_date === "9999-12-31") {
        end_date = null;
    } 
    let start_date = new Date(item.start_date);
    console.log(end_date);
    return (
        <>
            <SimpleGrid w="full" columns={{base: 2, md: 1}} gap="md">
                <GridItem> <p> {item.done? '済': '未'} </p></GridItem>
            
                <GridItem>
                    <div style={{textAlign:'right', fontSize:'0.7em'}}>
                        {item.update}
                    </div>
                </GridItem>
            </SimpleGrid>
            <p style={{textAlign:'center', fontSize:'1.1em'}}><strong>{item.title}</strong></p>
            <p>{item.work}</p>
            <div style={{fontSize:'0.9em'}}>
                <p>{start_date?.toLocaleDateString()} 〜 {end_date?.toLocaleDateString()}</p>
            </div>
        </>
    );
}

