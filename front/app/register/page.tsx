import { Suspense } from 'react'
import RegisterForm from './_components/register-form'

export default function RegisterPage() {
  return (
    <main className='flex flex-1 items-center justify-center p-4'>
      <Suspense>
        <RegisterForm />
      </Suspense>
    </main>
  )
}
