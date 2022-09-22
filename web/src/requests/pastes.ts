export async function getRoot() {
    let res = await fetch("http://localhost:3000/");
    let resp = await res.text();
    console.log(resp);
    return resp;
}
