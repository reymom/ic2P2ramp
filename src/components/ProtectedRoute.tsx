import { Navigate } from 'react-router-dom';
import { useUser } from './user/UserContext';
import { userTypeToString } from '../model/utils';

export type ProtectedRouteProps = {
    allowedUserTypes: string[];
    outlet: JSX.Element;
};

export default function ProtectedRoute({ allowedUserTypes, outlet }: ProtectedRouteProps) {
    const { user } = useUser();

    if (user && allowedUserTypes.includes(userTypeToString(user.user_type))) {
        return outlet;
    } else {
        return <Navigate to="/" />;
    }
}