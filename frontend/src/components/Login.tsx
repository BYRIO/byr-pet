export default function Component() {
    return (
        <>
            <div className="flex min-h-screen flex-col items-center justify-center bg-gray-100 px-4 py-12 dark:bg-gray-950">
                <div className="w-full max-w-md space-y-8">
                    <div className="text-center">
                        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-gray-50">
                            Welcome to BYR-pet
                        </h1>
                        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            输入学号和校园网密码以连接 BUPT-portal
                        </p>
                    </div>
                    <form className="space-y-6" action="/login" method="post">
                        <div>
                            <label
                                htmlFor="username"
                                className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                            >
                                学号
                            </label>
                            <div className="mt-1">
                                <input
                                    id="username"
                                    autoComplete="username"
                                    required={true}
                                    className="block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 placeholder-gray-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-50 dark:placeholder-gray-500"
                                    type="text"
                                    name="username"
                                />
                            </div>
                        </div>
                        <div>
                            <label
                                htmlFor="password"
                                className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                            >
                                密码
                            </label>
                            <div className="mt-1">
                                <input
                                    id="password"
                                    autoComplete="current-password"
                                    required={true}
                                    className="block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 placeholder-gray-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-50 dark:placeholder-gray-500"
                                    type="password"
                                    name="password"
                                />
                            </div>
                        </div>
                        <div>
                            <button
                                type="submit"
                                className="flex w-full justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-600 dark:focus:ring-indigo-600"
                            >
                                连接 BUPT-portal
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </>


    )
}