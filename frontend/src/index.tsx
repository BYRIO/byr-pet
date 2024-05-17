import { render } from 'preact';
import Login from './components/Login';
import './style.css';

export function App() {
	return (
		<Login />
	);
}

render(<App />, document.getElementById('app'));
