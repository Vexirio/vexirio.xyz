async function fetchSystemData() {
    try {
        const response = await fetch('http://95.57.7.43:3000/system');
        if (!response.ok) {
            throw new Error(`HTTP ошибка: ${response.status}`);
        }

        const data = await response.json();

        document.getElementById('total_memory').textContent = formatBytes(data.total_memory);
        document.getElementById('used_memory').textContent = formatBytes(data.used_memory);

        const networksList = document.getElementById('network_list');
        networksList.innerHTML = '';
        data.networks?.forEach(network => {
            const li = document.createElement('li');
            li.textContent = `${network.interface_name}: ${formatBytes(network.total_received)} ↓ / ${formatBytes(network.total_transmitted)} ↑`;
            networksList.appendChild(li);
        });

        const componentsList = document.getElementById('components_list');
        componentsList.innerHTML = '';
        data.components?.forEach(component => {
            const li = document.createElement('li');
            li.textContent = component;
            componentsList.appendChild(li);
        });

        const processList = document.getElementById('process_list');
        processList.innerHTML = '';
        data.processes?.forEach(proc => {
            const li = document.createElement('li');
            li.textContent = `PID ${proc.pid}: ${proc.name}`;
            processList.appendChild(li);
        });

    } catch (error) {
        console.error('Ошибка при загрузке данных системы:', error);
    }
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(2) + ' ' + sizes[i];
}

window.onload = () => {
    fetchSystemData(); // первый вызов при загрузке
    setInterval(fetchSystemData, 5000); // обновление каждые 5 секунд
};
