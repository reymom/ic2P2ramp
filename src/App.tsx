import { useEffect } from 'react';
import { Route, Routes } from 'react-router-dom';
import '@rainbow-me/rainbowkit/styles.css';
import 'react-json-view-lite/dist/index.css'; // JSON viewer component

import { useUser } from './components/user/UserContext';
import ProtectedRoute from './components/ProtectedRoute';
import Menu from './components/Menu';
import UserProfile from './components/user/UserProfile';
import ConnectAddress from './components/ConnectAddress';
import RegisterUser from './components/user/RegisterUser';
import CreateOrder from './components/order/CreateOrder';
import ViewOrders from './components/order/ViewOrders';
import { userTypeToString } from './model/utils';
import { OrderFilter } from './declarations/backend/backend.did';
import ConfirmEmail from './components/user/ConfirmEmail';
import ResetPassword from './components/user/ResetPassword';
import ForgotPassword from './components/user/ForgotPassword';

function App() {
    const { user } = useUser();

    useEffect(() => {
        if (user) {
            getInitialOrderFilter();
        }
    }, [user]);

    useEffect(() => {
        if (typeof window.Telegram !== 'undefined' && window.Telegram.WebApp) {
            const tg = window.Telegram.WebApp;

            tg.ready();
            tg.expand();
        }
    }, []);

    const getInitialOrderFilter = (): OrderFilter | null => {
        if (!user) return { ByState: { Created: null } };

        switch (userTypeToString(user.user_type)) {
            case "Offramper":
                return { ByOfframperId: user.id } as OrderFilter
            case "Onramper":
                return { ByOnramperId: user.id } as OrderFilter
            default:
                return { ByState: { Created: null } } as OrderFilter
        }
    }

    return (
        <div className="min-h-screen bg-gray-50">
            <Menu />
            <div className="flex-grow flex justify-center items-center py-8">
                <div className="text-center w-full sm:w-3/4 md:w-1/2 lg:w-5/12 xl:w-1/3">
                    <Routes>
                        <Route path="/" element={<ConnectAddress />} />
                        <Route path="/register" element={<RegisterUser />} />
                        <Route path="/confirm-email" element={<ConfirmEmail />} />
                        <Route path="/forgot-password" element={<ForgotPassword />} />
                        <Route path="/reset-password" element={<ResetPassword />} />
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
