function timeout(mill) {
    return new Promise(resolve => {
        setTimeout(() => {
            resolve();
        }, mill)
    })
}

export async function sleep(mill) {
    await timeout(mill);
}