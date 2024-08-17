import { useEffect } from 'react';
import { Route, Routes, Link } from 'react-router-dom';
import '@rainbow-me/rainbowkit/styles.css';
import 'react-json-view-lite/dist/index.css'; // JSON viewer component

import { useUser } from './UserContext';
import ProtectedRoute from './components/ProtectedRoute';
import Menu from './components/Menu';
import UserProfile from './components/UserProfile';
import ConnectAddress from './components/ConnectAddress';
import RegisterUser from './components/RegisterUser';
import CreateOrder from './components/order/CreateOrder';
import ViewOrders from './components/order/ViewOrders';
import { userTypeToString } from './model/utils';
import { OrderFilter } from './declarations/backend/backend.did';

function App() {
    const { user } = useUser();

    useEffect(() => {
        if (user) {
            getInitialOrderFilter();
        }
    }, [user]);

    const getInitialOrderFilter = (): OrderFilter | null => {
        if (!user) return { ByState: { Created: null } };

        switch (userTypeToString(user.user_type)) {
            case "Offramper":
                return { ByOfframperAddress: user.addresses[0] } as OrderFilter
            case "Onramper":
                return { LockedByOnramper: user.addresses[0] } as OrderFilter
            default:
                return { ByState: { Created: null } } as OrderFilter
        }
    }

    return (
        <div className="min-h-screen bg-gray-50">
            <Menu />
            <div className="flex flex-col items-center mt-8">
                <div className="bg-white p-4 rounded shadow-md text-center w-full sm:w-3/4 md:w-1/2 lg:w-1/3">
                    <Routes>
                        <Route path="/" element={<ConnectAddress />} />
                        <Route path="/login" element={<RegisterUser />} />
                        <Route
                            path="/create"
                            element={<ProtectedRoute allowedUserTypes={["Offramper"]} outlet={<CreateOrder />} />}
                        />
                        <Route path="/view" element={<ViewOrders initialFilter={getInitialOrderFilter()} />} />
                        <Route path="/profile" element={<UserProfile />} />
                    </Routes>
                </div>

            </div>
        </div>
    );
}

export default App;
