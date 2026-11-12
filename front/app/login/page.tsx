import LanguageSwitcher from '@/components/language-switcher'
import LoginForm from './_components/login-form'

export default function LoginPage() {
  return (
    <main className='relative flex flex-1 items-center justify-center p-4'>
      <div className='absolute top-2 right-4'>
        <LanguageSwitcher />
      </div>
      <LoginForm />
    </main>
  )
}
