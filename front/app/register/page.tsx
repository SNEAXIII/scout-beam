import LanguageSwitcher from '@/components/language-switcher'
import RegisterForm from './_components/register-form'

export default function RegisterPage() {
  return (
    <main className='relative flex flex-1 items-center justify-center p-4'>
      <div className='absolute top-2 right-4'>
        <LanguageSwitcher />
      </div>
      <RegisterForm />
    </main>
  )
}
