import { useState } from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';

import logo from './assets/p2ploan.webp';
import RegisterUser from './components/RegisterUser';
import CreateOrder from './components/CreateOrder';
import ViewOrders from './components/ViewOrders';
import ConnectAddress from './components/ConnectAddress';
import { pageTypes, userTypes } from './model/types';

// JSON viewer component
import 'react-json-view-lite/dist/index.css';

function App() {
    const [loading, setLoading] = useState(false);
    const [currentTab, setCurrentTab] = useState<pageTypes>(pageTypes.connect);
    const [selectedCurrency, setSelectedCurrency] = useState<string>('USD');
    const [userType, setUserType] = useState<userTypes>(userTypes.visitor);

    const handleCurrencyChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setSelectedCurrency(event.target.value);
    };

    return (
        <div className="min-h-screen bg-gray-100">
            <nav className="bg-white p-4 shadow-md flex justify-between items-center">
                <div className="flex items-center">
                    <img src={logo} className="rounded-full h-12 w-12 mr-2" alt="ic2P2ramp logo" />
                    <h1 className="text-xl font-bold">ic2P2ramp</h1>
                </div>
                <div className="flex items-center">
                    <select
                        value={selectedCurrency}
                        onChange={handleCurrencyChange}
                        className="px-4 py-2 border rounded mr-4"
                    >
                        <option value="USD">$</option>
                        <option value="EUR">€</option>
                        <option value="GBP">£</option>
                        <option value="JPY">¥</option>
                        <option value="SGD">S$</option>
                    </select>
                    <ConnectButton />
                </div>
            </nav>

            <div className="flex flex-col items-center mt-8">
                {/* <div className="flex justify-center mb-4">
                    <button
                        onClick={() => setCurrentTab(pageTypes.create)}
                        className={`px-4 py-2 mx-2 rounded ${currentTab === pageTypes.create ? 'bg-blue-500 text-white' : 'bg-white text-blue-500'}`}
                    >
                        Create Order
                    </button>
                    <button
                        onClick={() => setCurrentTab(pageTypes.view)}
                        className={`px-4 py-2 mx-2 rounded ${currentTab === pageTypes.view ? 'bg-blue-500 text-white' : 'bg-white text-blue-500'}`}
                    >
                        View Orders
                    </button>
                </div> */}
                <div className="bg-white p-4 rounded shadow-md text-center w-full sm:w-3/4 md:w-1/2 lg:w-1/3" style={{ opacity: loading ? 0.5 : 1 }}>
                    {currentTab === pageTypes.connect && <ConnectAddress setCurrentTab={setCurrentTab} />}
                    {currentTab === pageTypes.login && <RegisterUser setCurrentTab={setCurrentTab} />}
                    {currentTab === pageTypes.addProvider && <RegisterUser setCurrentTab={setCurrentTab} />}
                    {currentTab === pageTypes.create && <CreateOrder selectedCurrency={selectedCurrency} />}
                    {currentTab === pageTypes.view && <ViewOrders />}
                </div>
            </div>
        </div>
    );
}

export default App;
